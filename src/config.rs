use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub terminal: TerminalConfig,
    pub font: FontConfig,
    pub colors: ColorConfig,
    pub keybindings: KeybindingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub width: u32,
    pub height: u32,
    pub opacity: f32,
    pub decorations: bool,
    pub startup_mode: StartupMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub scrollback_lines: u32,
    pub shell: Option<String>,
    pub working_directory: Option<PathBuf>,
    pub cursor_blink: bool,
    pub cursor_shape: CursorShape,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: f32,
    pub bold_family: Option<String>,
    pub italic_family: Option<String>,
    pub bold_italic_family: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    pub selection_background: String,
    pub selection_foreground: String,
    pub normal: [String; 8],
    pub bright: [String; 8],
    pub dim: [String; 8],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    pub copy: String,
    pub paste: String,
    pub search: String,
    pub new_tab: String,
    pub close_tab: String,
    pub next_tab: String,
    pub prev_tab: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StartupMode {
    Windowed,
    Maximized,
    Fullscreen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CursorShape {
    Block,
    Underline,
    Beam,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            terminal: TerminalConfig::default(),
            font: FontConfig::default(),
            colors: ColorConfig::default(),
            keybindings: KeybindingConfig::default(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            opacity: 1.0,
            decorations: true,
            startup_mode: StartupMode::Windowed,
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: 10000,
            shell: None,
            working_directory: None,
            cursor_blink: true,
            cursor_shape: CursorShape::Block,
        }
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "monospace".to_string(),
            size: 12.0,
            bold_family: None,
            italic_family: None,
            bold_italic_family: None,
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            foreground: "#ffffff".to_string(),
            background: "#000000".to_string(),
            cursor: "#ffffff".to_string(),
            selection_background: "#444444".to_string(),
            selection_foreground: "#ffffff".to_string(),
            normal: [
                "#000000".to_string(), // Black
                "#800000".to_string(), // Red
                "#008000".to_string(), // Green
                "#808000".to_string(), // Yellow
                "#000080".to_string(), // Blue
                "#800080".to_string(), // Magenta
                "#008080".to_string(), // Cyan
                "#c0c0c0".to_string(), // White
            ],
            bright: [
                "#808080".to_string(), // Bright Black
                "#ff0000".to_string(), // Bright Red
                "#00ff00".to_string(), // Bright Green
                "#ffff00".to_string(), // Bright Yellow
                "#0000ff".to_string(), // Bright Blue
                "#ff00ff".to_string(), // Bright Magenta
                "#00ffff".to_string(), // Bright Cyan
                "#ffffff".to_string(), // Bright White
            ],
            dim: [
                "#000000".to_string(), // Dim Black
                "#400000".to_string(), // Dim Red
                "#004000".to_string(), // Dim Green
                "#404000".to_string(), // Dim Yellow
                "#000040".to_string(), // Dim Blue
                "#400040".to_string(), // Dim Magenta
                "#004040".to_string(), // Dim Cyan
                "#606060".to_string(), // Dim White
            ],
        }
    }
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        Self {
            copy: "Ctrl+Shift+C".to_string(),
            paste: "Ctrl+Shift+V".to_string(),
            search: "Ctrl+Shift+F".to_string(),
            new_tab: "Ctrl+Shift+T".to_string(),
            close_tab: "Ctrl+Shift+W".to_string(),
            next_tab: "Ctrl+Tab".to_string(),
            prev_tab: "Ctrl+Shift+Tab".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }
        
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
            
        let config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;
            
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }
        
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
            
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;
            
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        path.push("myterm");
        path.push("config.toml");
        Ok(path)
    }
}

pub fn parse_color(color_str: &str) -> Result<rgb::RGB8> {
    if color_str.starts_with('#') {
        let hex = &color_str[1..];
        if hex.len() != 6 {
            return Err(anyhow::anyhow!("Invalid color format: {}", color_str));
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;
        
        Ok(rgb::RGB8::new(r, g, b))
    } else {
        Err(anyhow::anyhow!("Unsupported color format: {}", color_str))
    }
}