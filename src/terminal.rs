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
    pub state: TerminalState,
    pub parser: Parser,
}

pub struct TerminalState {
    pub rows: usize,
    pub cols: usize,
    pub grid: Vec<Cell>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub _title: String,
    pub current_fg: Color,
    pub current_bg: Color,
    pub scroll_offset: usize,
    pub is_dirty: bool,
}

impl Terminal {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            state: TerminalState::new(rows, cols),
            parser: Parser::new(),
        }
    }

    pub fn advance(&mut self, bytes: &[u8]) {
        let mut handler = TerminalHandler {
            state: &mut self.state,
        };
        self.parser.advance(&mut handler, bytes);
        self.state.is_dirty = true;
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.state.resize(rows, cols);
    }
}

impl TerminalState {
    pub fn new(rows: usize, cols: usize) -> Self {
        let grid = vec![Cell::default(); rows * cols];
        Self {
            rows,
            cols,
            grid,
            cursor_row: 0,
            cursor_col: 0,
            _title: "myterm".to_string(),
            current_fg: Color(255, 255, 255, 255),
            current_bg: Color(0, 0, 0, 0),
            scroll_offset: 0,
            is_dirty: true,
        }
    }

    pub fn get_cell_mut(&mut self, row: usize, col: usize) -> &mut Cell {
        let actual_row = (row + self.scroll_offset) % self.rows;
        &mut self.grid[actual_row * self.cols + col]
    }

    pub fn get_cell(&self, row: usize, col: usize) -> &Cell {
        let actual_row = (row + self.scroll_offset) % self.rows;
        &self.grid[actual_row * self.cols + col]
    }

    pub fn scroll_up(&mut self) {
        let new_row_start = (self.scroll_offset) % self.rows;
        for col in 0..self.cols {
            self.grid[new_row_start * self.cols + col] = Cell::default();
        }
        self.scroll_offset = (self.scroll_offset + 1) % self.rows;
        self.is_dirty = true;
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        let mut new_grid = vec![Cell::default(); rows * cols];
        let min_rows = rows.min(self.rows);
        let min_cols = cols.min(self.cols);
        for r in 0..min_rows {
            for c in 0..min_cols {
                new_grid[r * cols + c] = self.get_cell(r, c).clone();
            }
        }
        self.rows = rows;
        self.cols = cols;
        self.grid = new_grid;
        self.scroll_offset = 0;
        self.cursor_row = self.cursor_row.min(rows - 1);
        self.cursor_col = self.cursor_col.min(cols - 1);
        self.is_dirty = true;
    }
}

struct TerminalHandler<'a> {
    state: &'a mut TerminalState,
}

impl<'a> Perform for TerminalHandler<'a> {
    fn print(&mut self, c: char) {
        let r = self.state.cursor_row;
        let c_col = self.state.cursor_col;
        if r >= self.state.rows || c_col >= self.state.cols {
            return;
        }
        *self.state.get_cell_mut(r, c_col) = Cell {
            c,
            fg: self.state.current_fg,
            bg: self.state.current_bg,
        };
        self.state.cursor_col += 1;
        if self.state.cursor_col >= self.state.cols {
            self.state.cursor_col = 0;
            self.state.cursor_row += 1;
            if self.state.cursor_row >= self.state.rows {
                self.state.cursor_row = self.state.rows - 1;
                self.state.scroll_up();
            }
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.state.cursor_row += 1;
                if self.state.cursor_row >= self.state.rows {
                    self.state.cursor_row = self.state.rows - 1;
                    self.state.scroll_up();
                }
            }
            b'\r' => {
                self.state.cursor_col = 0;
            }
            b'\x08' => {
                if self.state.cursor_col > 0 {
                    self.state.cursor_col -= 1;
                }
            }
            b'\t' => {
                let next_tab = (self.state.cursor_col + 8) & !7;
                self.state.cursor_col = next_tab.min(self.state.cols - 1);
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
                            self.state.current_fg = Color(255, 255, 255, 255);
                            self.state.current_bg = Color(0, 0, 0, 0);
                        }
                        _ => {}
                    }
                }
            }
            'H' | 'f' => {
                let mut iter = params.iter();
                let row = iter.next().and_then(|p| p.first()).cloned().unwrap_or(1) as usize;
                let col = iter.next().and_then(|p| p.first()).cloned().unwrap_or(1) as usize;
                self.state.cursor_row = (row.saturating_sub(1)).min(self.state.rows - 1);
                self.state.cursor_col = (col.saturating_sub(1)).min(self.state.cols - 1);
            }
            'J' => {
                let mode = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(0);
                match mode {
                    2 => {
                        for cell in self.state.grid.iter_mut() {
                            *cell = Cell::default();
                        }
                        self.state.cursor_row = 0;
                        self.state.cursor_col = 0;
                        self.state.scroll_offset = 0;
                    }
                    _ => {}
                }
            }
            'K' => {
                 let mode = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(0);
                 let row = self.state.cursor_row;
                 match mode {
                     0 => {
                        for col in self.state.cursor_col..self.state.cols {
                            *self.state.get_cell_mut(row, col) = Cell::default();
                        }
                     }
                     1 => {
                        for col in 0..=self.state.cursor_col {
                            *self.state.get_cell_mut(row, col) = Cell::default();
                        }
                     }
                     2 => {
                        for col in 0..self.state.cols {
                            *self.state.get_cell_mut(row, col) = Cell::default();
                        }
                     }
                     _ => {}
                 }
            }
            _ => {}
        }
    }
}
