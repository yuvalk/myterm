use myterm::config::Config;
use myterm::terminal::{Cell, CellFlags, Grid};

#[test]
fn test_cell_default() {
    let cell = Cell::default();
    assert_eq!(cell.c, ' ');
    assert_eq!(cell.flags, CellFlags::empty());
}

#[test]
fn test_cell_flags() {
    let mut flags = CellFlags::empty();
    assert!(!flags.contains(CellFlags::BOLD));
    
    flags.insert(CellFlags::BOLD);
    assert!(flags.contains(CellFlags::BOLD));
    
    flags.insert(CellFlags::ITALIC);
    assert!(flags.contains(CellFlags::BOLD | CellFlags::ITALIC));
    
    flags.remove(CellFlags::BOLD);
    assert!(!flags.contains(CellFlags::BOLD));
    assert!(flags.contains(CellFlags::ITALIC));
}

#[test]
fn test_grid_creation() {
    let grid = Grid::new(24, 80, 1000);
    assert_eq!(grid.rows, 24);
    assert_eq!(grid.cols, 80);
    assert_eq!(grid.scrollback_limit, 1000);
    assert_eq!(grid.cells.len(), 24);
    assert_eq!(grid.cells[0].len(), 80);
    
    // Check all cells are default
    for row in &grid.cells {
        for cell in row {
            assert_eq!(cell.c, ' ');
            assert_eq!(cell.flags, CellFlags::empty());
        }
    }
}

#[test]
fn test_grid_resize() {
    let mut grid = Grid::new(24, 80, 1000);
    
    // Resize to larger
    grid.resize(30, 120);
    assert_eq!(grid.rows, 30);
    assert_eq!(grid.cols, 120);
    assert_eq!(grid.cells.len(), 30);
    assert_eq!(grid.cells[0].len(), 120);
    
    // Resize to smaller
    grid.resize(20, 60);
    assert_eq!(grid.rows, 20);
    assert_eq!(grid.cols, 60);
    assert_eq!(grid.cells.len(), 20);
    assert_eq!(grid.cells[0].len(), 60);
}

#[test]
fn test_grid_scroll_up() {
    let mut grid = Grid::new(3, 3, 10);
    
    // Fill first row with 'A', second with 'B', third with 'C'
    for col in 0..3 {
        grid.cells[0][col].c = 'A';
        grid.cells[1][col].c = 'B';
        grid.cells[2][col].c = 'C';
    }
    
    grid.scroll_up(1);
    
    // First row should now be 'B', second 'C', third default
    for col in 0..3 {
        assert_eq!(grid.cells[0][col].c, 'B');
        assert_eq!(grid.cells[1][col].c, 'C');
        assert_eq!(grid.cells[2][col].c, ' ');
    }
    
    // Scrollback should contain the original first row
    assert_eq!(grid.scrollback.len(), 1);
    for col in 0..3 {
        assert_eq!(grid.scrollback[0][col].c, 'A');
    }
}

#[test]
fn test_grid_clear() {
    let mut grid = Grid::new(3, 3, 10);
    
    // Fill with some data
    for row in 0..3 {
        for col in 0..3 {
            grid.cells[row][col].c = 'X';
            grid.cells[row][col].flags = CellFlags::BOLD;
        }
    }
    
    grid.clear();
    
    // All cells should be default again
    for row in &grid.cells {
        for cell in row {
            assert_eq!(cell.c, ' ');
            assert_eq!(cell.flags, CellFlags::empty());
        }
    }
}

#[test]
fn test_grid_clear_line() {
    let mut grid = Grid::new(3, 3, 10);
    
    // Fill with some data
    for row in 0..3 {
        for col in 0..3 {
            grid.cells[row][col].c = 'X';
        }
    }
    
    grid.clear_line(1);
    
    // Only middle row should be cleared
    for col in 0..3 {
        assert_eq!(grid.cells[0][col].c, 'X');
        assert_eq!(grid.cells[1][col].c, ' ');
        assert_eq!(grid.cells[2][col].c, 'X');
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_terminal_creation() {
        let config = Config::default();
        let terminal = myterm::terminal::Terminal::new(&config);
        assert!(terminal.is_ok());
    }
    
    #[tokio::test] 
    async fn test_terminal_resize() {
        let config = Config::default();
        let mut terminal = myterm::terminal::Terminal::new(&config).unwrap();
        
        let result = terminal.resize(1024, 768);
        assert!(result.is_ok());
    }
}