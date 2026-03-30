use vte::{Parser, Perform};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: Color(255, 255, 255, 255),
            bg: Color(0, 0, 0, 0),
        }
    }
}

pub struct Terminal {
    pub rows: usize,
    pub cols: usize,
    pub grid: Vec<Vec<Cell>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub parser: Parser,
    pub _title: String,
    pub current_fg: Color,
    pub current_bg: Color,
}

impl Terminal {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut grid = Vec::with_capacity(rows);
        for _ in 0..rows {
            grid.push(vec![Cell::default(); cols]);
        }
        Self {
            rows,
            cols,
            grid,
            cursor_row: 0,
            cursor_col: 0,
            parser: Parser::new(),
            _title: "myterm".to_string(),
            current_fg: Color(255, 255, 255, 255),
            current_bg: Color(0, 0, 0, 0),
        }
    }

    pub fn advance(&mut self, byte: u8) {
        let mut handler = TerminalHandler {
            grid: &mut self.grid,
            cursor_row: &mut self.cursor_row,
            cursor_col: &mut self.cursor_col,
            rows: self.rows,
            cols: self.cols,
            current_fg: &mut self.current_fg,
            current_bg: &mut self.current_bg,
        };
        self.parser.advance(&mut handler, &[byte]);
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.rows = rows;
        self.cols = cols;
        self.grid.resize(rows, vec![Cell::default(); cols]);
        for row in self.grid.iter_mut() {
            row.resize(cols, Cell::default());
        }
        if self.cursor_row >= rows {
            self.cursor_row = rows - 1;
        }
        if self.cursor_col >= cols {
            self.cursor_col = cols - 1;
        }
    }
}

struct TerminalHandler<'a> {
    grid: &'a mut Vec<Vec<Cell>>,
    cursor_row: &'a mut usize,
    cursor_col: &'a mut usize,
    rows: usize,
    cols: usize,
    current_fg: &'a mut Color,
    current_bg: &'a mut Color,
}

impl<'a> Perform for TerminalHandler<'a> {
    fn print(&mut self, c: char) {
        if *self.cursor_row >= self.rows || *self.cursor_col >= self.cols {
            return;
        }
        self.grid[*self.cursor_row][*self.cursor_col] = Cell {
            c,
            fg: *self.current_fg,
            bg: *self.current_bg,
        };
        *self.cursor_col += 1;
        if *self.cursor_col >= self.cols {
            *self.cursor_col = 0;
            *self.cursor_row += 1;
            if *self.cursor_row >= self.rows {
                *self.cursor_row = self.rows - 1;
                self.grid.remove(0);
                self.grid.push(vec![Cell::default(); self.cols]);
            }
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                *self.cursor_row += 1;
                if *self.cursor_row >= self.rows {
                    *self.cursor_row = self.rows - 1;
                    self.grid.remove(0);
                    self.grid.push(vec![Cell::default(); self.cols]);
                }
            }
            b'\r' => {
                *self.cursor_col = 0;
            }
            b'\x08' => {
                if *self.cursor_col > 0 {
                    *self.cursor_col -= 1;
                }
            }
            b'\t' => {
                let next_tab = (*self.cursor_col + 8) & !7;
                *self.cursor_col = next_tab.min(self.cols - 1);
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &vte::Params, _intermediates: &[u8], _ignore: bool, action: char) {
        match action {
            'm' => {
                for param in params {
                    match param {
                        [0] => {
                            *self.current_fg = Color(255, 255, 255, 255);
                            *self.current_bg = Color(0, 0, 0, 0);
                        }
                        _ => {}
                    }
                }
            }
            'H' | 'f' => {
                let mut iter = params.iter();
                let row = iter.next().and_then(|p| p.first()).cloned().unwrap_or(1) as usize;
                let col = iter.next().and_then(|p| p.first()).cloned().unwrap_or(1) as usize;
                *self.cursor_row = (row.saturating_sub(1)).min(self.rows - 1);
                *self.cursor_col = (col.saturating_sub(1)).min(self.cols - 1);
            }
            'J' => {
                let mode = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(0);
                match mode {
                    2 => {
                        for row in self.grid.iter_mut() {
                            for cell in row.iter_mut() {
                                *cell = Cell::default();
                            }
                        }
                        *self.cursor_row = 0;
                        *self.cursor_col = 0;
                    }
                    _ => {}
                }
            }
            'K' => {
                 let mode = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(0);
                 match mode {
                     0 => {
                        for col in *self.cursor_col..self.cols {
                            self.grid[*self.cursor_row][col] = Cell::default();
                        }
                     }
                     1 => {
                        for col in 0..=*self.cursor_col {
                            self.grid[*self.cursor_row][col] = Cell::default();
                        }
                     }
                     2 => {
                        for cell in self.grid[*self.cursor_row].iter_mut() {
                            *cell = Cell::default();
                        }
                     }
                     _ => {}
                 }
            }
            _ => {}
        }
    }
}