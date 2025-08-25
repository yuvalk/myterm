# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MyTerm is a modern terminal emulator built in Rust specifically for Sway window manager and Wayland protocol. It provides VT100/VT220/xterm compatibility with hardware-accelerated rendering and native Wayland integration.

## Development Commands

### Building and Testing
```bash
# Build debug version
cargo build

# Build release version  
cargo build --release
# or
make release

# Run tests
cargo test --all-features
# or  
make test

# Run tests with coverage
make test-coverage

# Run benchmarks
make bench
```

### Code Quality
```bash
# Run clippy linting (required before commits)
cargo clippy --all-targets --all-features -- -D warnings
# or
make clippy

# Format code
cargo fmt --all
# or
make format

# Check formatting
make format-check

# Run security audit
make audit
```

### Development Setup
```bash
# Install development tools
make dev
```

## Architecture Overview

The codebase follows a modular architecture with clear separation of concerns:

### Core Modules

- **`src/main.rs`**: Application entry point with async event loop using tokio::select! for handling display events and terminal output concurrently
- **`src/config.rs`**: TOML-based configuration system with structured config types (DisplayConfig, TerminalConfig, FontConfig, ColorConfig, KeybindingConfig)
- **`src/terminal.rs`**: Terminal emulation core using VTE parser with Grid/Cell model for text buffer and ANSI escape sequence processing
- **`src/display.rs`**: Wayland display management and rendering coordination
- **`src/wayland.rs`**: Wayland protocol implementation using smithay-client-toolkit
- **`src/pty.rs`**: Pseudoterminal management for shell interaction
- **`src/input.rs`**: Keyboard and mouse input handling

### Key Design Patterns

- **Event-driven architecture**: Main loop uses `tokio::select!` to handle display events and terminal output concurrently
- **Channel-based communication**: Uses crossbeam-channel for inter-thread communication between PTY and terminal
- **VTE-based parsing**: Terminal emulation built on the VTE crate for ANSI escape sequence processing
- **Wayland-native**: Built specifically for Wayland using smithay-client-toolkit, not a generic terminal with Wayland support

### Configuration System

Configuration files are loaded from:
- `~/.config/myterm/config.toml`
- `$XDG_CONFIG_HOME/myterm/config.toml`

All configuration is strongly typed using serde for TOML parsing with comprehensive defaults.

## Dependencies and Libraries

### Core Dependencies
- **wayland-client/wayland-protocols**: Wayland protocol implementation
- **smithay-client-toolkit**: High-level Wayland client toolkit
- **vte**: Terminal emulation and ANSI parsing
- **tokio**: Async runtime for event handling
- **crossbeam-channel**: Lock-free channels for communication

### Rendering and Fonts
- **fontconfig/freetype/harfbuzz_rs**: Font loading and text shaping
- **unicode-width**: Unicode character width calculations

### Configuration and Serialization  
- **serde/toml**: Configuration file parsing
- **anyhow/thiserror**: Error handling

## Testing

Tests are organized by module:
- `tests/config_tests.rs`: Configuration loading and validation
- `tests/input_tests.rs`: Input handling and key mapping
- `tests/terminal_tests.rs`: Terminal emulation and VTE integration
- `benches/terminal_benchmark.rs`: Performance benchmarking

## Important Implementation Details

- Terminal grid uses `Vec<Vec<Cell>>` structure with `Cell` containing character, colors, and text attributes
- Cursor state includes position, shape (Block/Bar/Underline), and visibility
- Color system supports 24-bit RGB with both normal and bright color variants
- Font rendering supports multiple font families for regular/bold/italic variants
- Input system converts Wayland keyboard events to terminal byte sequences
- PTY communication is asynchronous with proper signal handling

## Common Development Tasks

When working with this codebase:

1. **Adding new configuration options**: Extend the appropriate config struct in `src/config.rs` and update the default implementation
2. **Modifying terminal behavior**: Terminal emulation logic is in `src/terminal.rs` with VTE integration
3. **Adding display features**: Display and rendering logic is in `src/display.rs` and `src/wayland.rs`
4. **Input handling changes**: Keyboard/mouse input processing is in `src/input.rs`

Always run `make clippy` and `make test` before committing changes.