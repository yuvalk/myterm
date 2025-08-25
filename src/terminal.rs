use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::VecDeque;
use vte::{Perform, Parser};

use crate::config::{Config, CursorShape};
use crate::pty::Pty;

pub struct Terminal {
    pty: Pty,
    parser: Parser,
    performer: TerminalPerformer,
    #[allow(dead_code)]
    output_receiver: Receiver<Vec<u8>>,
    #[allow(dead_code)]
    input_sender: Sender<Vec<u8>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Cell {
    pub c: char,
    pub fg: rgb::RGB8,
    pub bg: rgb::RGB8,
    pub flags: CellFlags,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CellFlags: u8 {
        const BOLD = 0b00000001;
        const DIM = 0b00000010;
        const ITALIC = 0b00000100;
        const UNDERLINE = 0b00001000;
        const STRIKETHROUGH = 0b00010000;
        const REVERSE = 0b00100000;
        const BLINK = 0b01000000;
        const HIDDEN = 0b10000000;
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub shape: CursorShape,
    pub visible: bool,
}

pub struct Grid {
    pub cells: Vec<Vec<Cell>>,
    pub rows: usize,
    pub cols: usize,
    pub scrollback: VecDeque<Vec<Cell>>,
    pub scrollback_limit: usize,
}

pub struct TerminalPerformer {
    pub grid: Grid,
    pub cursor: Cursor,
    pub default_fg: rgb::RGB8,
    pub default_bg: rgb::RGB8,
    pub current_fg: rgb::RGB8,
    pub current_bg: rgb::RGB8,
    pub current_flags: CellFlags,
    #[allow(dead_code)]
    pub saved_cursor: Option<Cursor>,
    pub scroll_region: (usize, usize),
    pub insert_mode: bool,
    pub auto_wrap_mode: bool,
    #[allow(dead_code)]
    pub origin_mode: bool,
    pub title: String,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: rgb::RGB8::new(255, 255, 255),
            bg: rgb::RGB8::new(0, 0, 0),
            flags: CellFlags::empty(),
        }
    }
}

impl Grid {
    pub fn new(rows: usize, cols: usize, scrollback_limit: usize) -> Self {
        let cells = vec![vec![Cell::default(); cols]; rows];
        Self {
            cells,
            rows,
            cols,
            scrollback: VecDeque::with_capacity(scrollback_limit),
            scrollback_limit,
        }
    }
    
    pub fn resize(&mut self, new_rows: usize, new_cols: usize) {
        if new_cols != self.cols {
            for row in &mut self.cells {
                row.resize(new_cols, Cell::default());
            }
            self.cols = new_cols;
        }
        
        if new_rows != self.rows {
            self.cells.resize(new_rows, vec![Cell::default(); new_cols]);
            self.rows = new_rows;
        }
    }
    
    pub fn scroll_up(&mut self, lines: usize) {
        for _ in 0..lines {
            if self.scrollback.len() >= self.scrollback_limit {
                self.scrollback.pop_front();
            }
            
            let first_row = self.cells.remove(0);
            self.scrollback.push_back(first_row);
            self.cells.push(vec![Cell::default(); self.cols]);
        }
    }
    
    #[allow(dead_code)]
    pub fn scroll_down(&mut self, lines: usize) {
        for _ in 0..lines {
            if let Some(row) = self.scrollback.pop_back() {
                self.cells.insert(0, row);
                self.cells.pop();
            } else {
                self.cells.insert(0, vec![Cell::default(); self.cols]);
                self.cells.pop();
            }
        }
    }
    
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
    }
    
    pub fn clear_line(&mut self, row: usize) {
        if row < self.rows {
            for cell in &mut self.cells[row] {
                *cell = Cell::default();
            }
        }
    }
}

impl TerminalPerformer {
    pub fn new(rows: usize, cols: usize, config: &Config) -> Self {
        let default_fg = crate::config::parse_color(&config.colors.foreground).unwrap_or(rgb::RGB8::new(255, 255, 255));
        let default_bg = crate::config::parse_color(&config.colors.background).unwrap_or(rgb::RGB8::new(0, 0, 0));
        
        Self {
            grid: Grid::new(rows, cols, config.terminal.scrollback_lines as usize),
            cursor: Cursor {
                row: 0,
                col: 0,
                shape: config.terminal.cursor_shape.clone(),
                visible: true,
            },
            default_fg,
            default_bg,
            current_fg: default_fg,
            current_bg: default_bg,
            current_flags: CellFlags::empty(),
            saved_cursor: None,
            scroll_region: (0, rows.saturating_sub(1)),
            insert_mode: false,
            auto_wrap_mode: true,
            origin_mode: false,
            title: String::new(),
        }
    }
    
    fn put_char(&mut self, c: char) {
        if self.cursor.row >= self.grid.rows || self.cursor.col >= self.grid.cols {
            return;
        }
        
        let cell = Cell {
            c,
            fg: self.current_fg,
            bg: self.current_bg,
            flags: self.current_flags,
        };
        
        if self.insert_mode {
            self.grid.cells[self.cursor.row].insert(self.cursor.col, cell);
            if self.grid.cells[self.cursor.row].len() > self.grid.cols {
                self.grid.cells[self.cursor.row].truncate(self.grid.cols);
            }
        } else {
            self.grid.cells[self.cursor.row][self.cursor.col] = cell;
        }
        
        self.cursor.col += 1;
        
        if self.cursor.col >= self.grid.cols {
            if self.auto_wrap_mode {
                self.cursor.col = 0;
                self.cursor.row += 1;
                
                if self.cursor.row > self.scroll_region.1 {
                    self.grid.scroll_up(1);
                    self.cursor.row = self.scroll_region.1;
                }
            } else {
                self.cursor.col = self.grid.cols - 1;
            }
        }
    }
}

impl Perform for TerminalPerformer {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }
    
    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => { // Backspace
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                }
            }
            0x09 => { // Tab
                self.cursor.col = ((self.cursor.col / 8) + 1) * 8;
                if self.cursor.col >= self.grid.cols {
                    self.cursor.col = self.grid.cols - 1;
                }
            }
            0x0A => { // Line Feed
                self.cursor.row += 1;
                if self.cursor.row > self.scroll_region.1 {
                    self.grid.scroll_up(1);
                    self.cursor.row = self.scroll_region.1;
                }
            }
            0x0D => { // Carriage Return
                self.cursor.col = 0;
            }
            _ => {}
        }
    }
    
    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _c: char) {
    }
    
    fn put(&mut self, _byte: u8) {
    }
    
    fn unhook(&mut self) {
    }
    
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.len() >= 2 && params[0] == b"0" {
            if let Ok(title) = std::str::from_utf8(params[1]) {
                self.title = title.to_string();
            }
        }
    }
    
    fn csi_dispatch(&mut self, params: &vte::Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'A' => { // Cursor Up
                let n = params.iter().next().unwrap_or(&[1])[0].max(1) as usize;
                self.cursor.row = self.cursor.row.saturating_sub(n);
            }
            'B' => { // Cursor Down
                let n = params.iter().next().unwrap_or(&[1])[0].max(1) as usize;
                self.cursor.row = (self.cursor.row + n).min(self.grid.rows - 1);
            }
            'C' => { // Cursor Forward
                let n = params.iter().next().unwrap_or(&[1])[0].max(1) as usize;
                self.cursor.col = (self.cursor.col + n).min(self.grid.cols - 1);
            }
            'D' => { // Cursor Backward
                let n = params.iter().next().unwrap_or(&[1])[0].max(1) as usize;
                self.cursor.col = self.cursor.col.saturating_sub(n);
            }
            'H' | 'f' => { // Cursor Position
                let mut iter = params.iter();
                let row = iter.next().unwrap_or(&[1])[0].max(1) as usize - 1;
                let col = iter.next().unwrap_or(&[1])[0].max(1) as usize - 1;
                self.cursor.row = row.min(self.grid.rows - 1);
                self.cursor.col = col.min(self.grid.cols - 1);
            }
            'J' => { // Erase in Display
                let n = params.iter().next().unwrap_or(&[0])[0];
                match n {
                    0 => { // Clear from cursor to end of screen
                        for col in self.cursor.col..self.grid.cols {
                            self.grid.cells[self.cursor.row][col] = Cell::default();
                        }
                        for row in (self.cursor.row + 1)..self.grid.rows {
                            self.grid.clear_line(row);
                        }
                    }
                    1 => { // Clear from beginning of screen to cursor
                        for row in 0..self.cursor.row {
                            self.grid.clear_line(row);
                        }
                        for col in 0..=self.cursor.col {
                            self.grid.cells[self.cursor.row][col] = Cell::default();
                        }
                    }
                    2 => { // Clear entire screen
                        self.grid.clear();
                    }
                    _ => {}
                }
            }
            'K' => { // Erase in Line
                let n = params.iter().next().unwrap_or(&[0])[0];
                match n {
                    0 => { // Clear from cursor to end of line
                        for col in self.cursor.col..self.grid.cols {
                            self.grid.cells[self.cursor.row][col] = Cell::default();
                        }
                    }
                    1 => { // Clear from beginning of line to cursor
                        for col in 0..=self.cursor.col {
                            self.grid.cells[self.cursor.row][col] = Cell::default();
                        }
                    }
                    2 => { // Clear entire line
                        self.grid.clear_line(self.cursor.row);
                    }
                    _ => {}
                }
            }
            'm' => { // Set Graphics Rendition
                for param in params.iter() {
                    for &value in param {
                        match value {
                            0 => { // Reset
                                self.current_fg = self.default_fg;
                                self.current_bg = self.default_bg;
                                self.current_flags = CellFlags::empty();
                            }
                            1 => self.current_flags.insert(CellFlags::BOLD),
                            2 => self.current_flags.insert(CellFlags::DIM),
                            3 => self.current_flags.insert(CellFlags::ITALIC),
                            4 => self.current_flags.insert(CellFlags::UNDERLINE),
                            7 => self.current_flags.insert(CellFlags::REVERSE),
                            22 => self.current_flags.remove(CellFlags::BOLD | CellFlags::DIM),
                            23 => self.current_flags.remove(CellFlags::ITALIC),
                            24 => self.current_flags.remove(CellFlags::UNDERLINE),
                            27 => self.current_flags.remove(CellFlags::REVERSE),
                            30..=37 => {
                                let _color_index = (value - 30) as usize;
                                // Use default colors for now, proper color handling would go here
                                self.current_fg = self.default_fg;
                            }
                            40..=47 => {
                                let _color_index = (value - 40) as usize;
                                // Use default colors for now, proper color handling would go here
                                self.current_bg = self.default_bg;
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
    }
}

impl Terminal {
    pub fn new(config: &Config) -> Result<Self> {
        let pty = Pty::new()?;
        let parser = Parser::new();
        let performer = TerminalPerformer::new(24, 80, config); // Default size
        
        let (input_sender, _input_receiver) = unbounded();
        let (_output_sender, output_receiver) = unbounded();
        
        Ok(Self {
            pty,
            parser,
            performer,
            output_receiver,
            input_sender,
        })
    }
    
    pub async fn start_shell(&mut self, config: &Config) -> Result<()> {
        let shell = config.terminal.shell.as_deref();
        let working_dir = config.terminal.working_directory.as_ref().and_then(|p| p.to_str());
        
        self.pty.spawn_shell(shell, working_dir).await?;
        Ok(())
    }
    
    pub async fn write_to_pty(&mut self, data: &[u8]) -> Result<()> {
        self.pty.write(data).await
    }
    
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        let cols = (width / 8).max(1) as u16; // Rough estimation
        let rows = (height / 16).max(1) as u16; // Rough estimation
        
        self.pty.resize(cols, rows)?;
        self.performer.grid.resize(rows as usize, cols as usize);
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn handle_key(&mut self, _key: crate::input::Key) -> Result<()> {
        // Key handling implementation would go here
        Ok(())
    }
    
    pub async fn next_output(&mut self) -> Result<Option<Vec<u8>>> {
        let mut buf = vec![0u8; 4096];
        match self.pty.read(&mut buf).await {
            Ok(n) => {
                buf.truncate(n);
                
                // Parse the output through VTE
                for &byte in &buf {
                    self.parser.advance(&mut self.performer, byte);
                }
                
                Ok(Some(buf))
            }
            Err(_) => Ok(None),
        }
    }
    
    pub fn grid(&self) -> &Grid {
        &self.performer.grid
    }
    
    #[allow(dead_code)]
    pub fn cursor(&self) -> &Cursor {
        &self.performer.cursor
    }
    
    #[allow(dead_code)]
    pub fn title(&self) -> &str {
        &self.performer.title
    }
}