# MyTerm

A modern, fast terminal emulator designed for Sway window manager and Wayland protocol.

[![CI](https://github.com/yuvalk/myterm/workflows/CI/badge.svg)](https://github.com/yuvalk/myterm/actions)
[![codecov](https://codecov.io/gh/yuvalk/myterm/branch/main/graph/badge.svg)](https://codecov.io/gh/yuvalk/myterm)
[![Crates.io](https://img.shields.io/crates/v/myterm.svg)](https://crates.io/crates/myterm)

## Features

### Core Terminal Features
- **VT100/VT220/xterm compatibility** - Full ANSI escape sequence support
- **True color support** - 24-bit RGB color depth
- **Unicode support** - Complete UTF-8 and Unicode character rendering
- **Scrollback buffer** - Configurable history with search functionality
- **Copy/paste integration** - Native Wayland clipboard support

### Sway Integration
- **Native Wayland** - Built specifically for Wayland protocol
- **Sway window management** - Seamless integration with tiling capabilities
- **Workspace awareness** - Proper behavior across Sway workspaces
- **HiDPI support** - Automatic scaling on high-resolution displays

### Performance
- **GPU acceleration** - Hardware-accelerated rendering where available
- **Low memory footprint** - Efficient memory usage for long-running sessions
- **Fast startup** - Quick application launch times
- **Low input latency** - Responsive typing experience

### Customization
- **TOML configuration** - Easy-to-edit configuration files
- **Custom themes** - Configurable color schemes
- **Font configuration** - Support for multiple font families and sizes
- **Keybinding customization** - Configurable keyboard shortcuts

## Installation

### Prerequisites

MyTerm requires a Wayland compositor (like Sway) and the following system libraries:

#### Ubuntu/Debian
```bash
sudo apt install libwayland-dev libxkbcommon-dev libegl1-mesa-dev \
                 libfontconfig1-dev libfreetype6-dev libharfbuzz-dev
```

#### Fedora/RHEL
```bash
sudo dnf install wayland-devel libxkbcommon-devel mesa-libEGL-devel \
                 fontconfig-devel freetype-devel harfbuzz-devel
```

#### Arch Linux
```bash
sudo pacman -S wayland libxkbcommon mesa fontconfig freetype2 harfbuzz
```

### From Source

1. Install [Rust](https://rustup.rs/) if you haven't already
2. Clone the repository:
   ```bash
   git clone https://github.com/yuvalk/myterm.git
   cd myterm
   ```
3. Build and install:
   ```bash
   cargo build --release
   sudo cp target/release/myterm /usr/local/bin/
   ```

### Package Managers

#### From crates.io
```bash
cargo install myterm
```

#### Debian Package
```bash
# Download from releases page
wget https://github.com/yuvalk/myterm/releases/latest/download/myterm.deb
sudo dpkg -i myterm.deb
```

#### Arch Linux (AUR)
```bash
# Using an AUR helper (recommended)
yay -S myterm          # Stable release
yay -S myterm-git      # Latest development version

# Or with paru
paru -S myterm
paru -S myterm-git

# Manual installation
git clone https://aur.archlinux.org/myterm.git
cd myterm
makepkg -si
```

#### Flatpak
```bash
flatpak install flathub com.github.yuvalk.myterm
```

## Configuration

MyTerm looks for configuration files in the following locations:
- `~/.config/myterm/config.toml`
- `$XDG_CONFIG_HOME/myterm/config.toml`

### Example Configuration

```toml
[display]
width = 1024
height = 768
opacity = 0.95
decorations = true
startup_mode = "Windowed"

[terminal]
scrollback_lines = 10000
shell = "/bin/zsh"
cursor_blink = true
cursor_shape = "Block"

[font]
family = "Fira Code"
size = 12.0

[colors]
foreground = "#ffffff"
background = "#1e1e1e"
cursor = "#ffffff"

# Normal colors
normal = [
    "#000000", # Black
    "#cd3131", # Red
    "#0dbc79", # Green
    "#e5e510", # Yellow
    "#2472c8", # Blue
    "#bc3fbc", # Magenta
    "#11a8cd", # Cyan
    "#e5e5e5", # White
]

# Bright colors
bright = [
    "#666666", # Bright Black
    "#f14c4c", # Bright Red
    "#23d18b", # Bright Green
    "#f5f543", # Bright Yellow
    "#3b8eea", # Bright Blue
    "#d670d6", # Bright Magenta
    "#29b8db", # Bright Cyan
    "#ffffff", # Bright White
]

[keybindings]
copy = "Ctrl+Shift+C"
paste = "Ctrl+Shift+V"
search = "Ctrl+Shift+F"
new_tab = "Ctrl+Shift+T"
close_tab = "Ctrl+Shift+W"
```

## Usage

### Basic Usage
```bash
myterm                    # Launch with default settings
myterm --config /path/to/config.toml  # Use custom config
myterm --working-directory ~/projects  # Set working directory
```

### Keybindings (Default)

| Action | Keybinding |
|--------|------------|
| Copy | Ctrl+Shift+C |
| Paste | Ctrl+Shift+V |
| Search | Ctrl+Shift+F |
| New Tab | Ctrl+Shift+T |
| Close Tab | Ctrl+Shift+W |
| Next Tab | Ctrl+Tab |
| Previous Tab | Ctrl+Shift+Tab |

## Development

### Building from Source

1. Clone the repository
2. Install system dependencies (see Installation section)
3. Run development build:
   ```bash
   cargo build
   ```

### Testing

```bash
# Run tests
make test

# Run tests with coverage
make test-coverage

# Run benchmarks
make bench

# Run linting
make clippy
```

### Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run the test suite: `make test`
5. Run linting: `make clippy`
6. Commit your changes: `git commit -am 'Add feature'`
7. Push to the branch: `git push origin feature-name`
8. Create a Pull Request

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [smithay-client-toolkit](https://github.com/Smithay/client-toolkit) for Wayland integration
- Terminal emulation powered by [vte](https://crates.io/crates/vte)
- Font rendering with [FreeType](https://www.freetype.org/) and [HarfBuzz](https://harfbuzz.github.io/)

## Support

- Report bugs and request features on [GitHub Issues](https://github.com/yuvalk/myterm/issues)
- Discuss the project on [GitHub Discussions](https://github.com/yuvalk/myterm/discussions)
- For security issues, please email security@example.com