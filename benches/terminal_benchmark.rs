use criterion::{black_box, criterion_group, criterion_main, Criterion};
use myterm::config::Config;
use myterm::terminal::{Grid, Cell, CellFlags, TerminalPerformer};
use vte::Parser;

fn benchmark_grid_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid");
    
    group.bench_function("create_grid", |b| {
        b.iter(|| Grid::new(black_box(24), black_box(80), black_box(1000)))
    });
    
    group.bench_function("resize_grid", |b| {
        let mut grid = Grid::new(24, 80, 1000);
        b.iter(|| {
            grid.resize(black_box(30), black_box(120));
            grid.resize(black_box(24), black_box(80));
        })
    });
    
    group.bench_function("scroll_up", |b| {
        let mut grid = Grid::new(24, 80, 1000);
        // Fill grid with some data
        for row in 0..24 {
            for col in 0..80 {
                grid.cells[row][col].c = 'X';
            }
        }
        
        b.iter(|| {
            grid.scroll_up(black_box(1));
        })
    });
    
    group.bench_function("clear_grid", |b| {
        let mut grid = Grid::new(24, 80, 1000);
        // Fill grid with some data
        for row in 0..24 {
            for col in 0..80 {
                grid.cells[row][col].c = 'X';
                grid.cells[row][col].flags = CellFlags::BOLD;
            }
        }
        
        b.iter(|| {
            grid.clear();
        })
    });
    
    group.finish();
}

fn benchmark_vte_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("vte");
    
    let config = Config::default();
    let mut performer = TerminalPerformer::new(24, 80, &config);
    let mut parser = Parser::new();
    
    // Test data with ANSI escape sequences
    let test_data = b"\x1b[2J\x1b[H\x1b[31mHello, \x1b[32mWorld!\x1b[0m\n\r";
    
    group.bench_function("parse_ansi", |b| {
        b.iter(|| {
            for &byte in black_box(test_data) {
                parser.advance(&mut performer, byte);
            }
        })
    });
    
    // Benchmark parsing a large text block
    let large_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
    let large_text_bytes = large_text.as_bytes();
    
    group.bench_function("parse_large_text", |b| {
        b.iter(|| {
            for &byte in black_box(large_text_bytes) {
                parser.advance(&mut performer, byte);
            }
        })
    });
    
    group.finish();
}

fn benchmark_cell_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell");
    
    group.bench_function("create_cell", |b| {
        b.iter(|| Cell {
            c: black_box('A'),
            fg: black_box(rgb::RGB8::new(255, 255, 255)),
            bg: black_box(rgb::RGB8::new(0, 0, 0)),
            flags: black_box(CellFlags::BOLD),
        })
    });
    
    group.bench_function("clone_cell", |b| {
        let cell = Cell {
            c: 'A',
            fg: rgb::RGB8::new(255, 255, 255),
            bg: rgb::RGB8::new(0, 0, 0),
            flags: CellFlags::BOLD,
        };
        
        b.iter(|| black_box(&cell).clone())
    });
    
    group.bench_function("modify_cell_flags", |b| {
        let mut cell = Cell::default();
        
        b.iter(|| {
            cell.flags.insert(black_box(CellFlags::BOLD));
            cell.flags.insert(black_box(CellFlags::ITALIC));
            cell.flags.remove(black_box(CellFlags::BOLD));
            cell.flags = black_box(CellFlags::empty());
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_grid_operations,
    benchmark_vte_parsing,
    benchmark_cell_operations
);
criterion_main!(benches);