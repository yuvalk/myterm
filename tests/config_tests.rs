use myterm::config::{Config, parse_color};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_default_config() {
    let config = Config::default();
    
    assert_eq!(config.display.width, 800);
    assert_eq!(config.display.height, 600);
    assert_eq!(config.display.opacity, 1.0);
    assert_eq!(config.terminal.scrollback_lines, 10000);
    assert_eq!(config.font.family, "monospace");
    assert_eq!(config.font.size, 12.0);
}

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let toml_str = toml::to_string(&config).expect("Failed to serialize config");
    
    let deserialized: Config = toml::from_str(&toml_str)
        .expect("Failed to deserialize config");
    
    assert_eq!(config.display.width, deserialized.display.width);
    assert_eq!(config.terminal.scrollback_lines, deserialized.terminal.scrollback_lines);
    assert_eq!(config.font.family, deserialized.font.family);
}

#[test]
fn test_color_parsing() {
    // Test valid hex colors
    let white = parse_color("#ffffff").expect("Failed to parse white");
    assert_eq!(white.r, 255);
    assert_eq!(white.g, 255);
    assert_eq!(white.b, 255);
    
    let black = parse_color("#000000").expect("Failed to parse black");
    assert_eq!(black.r, 0);
    assert_eq!(black.g, 0);
    assert_eq!(black.b, 0);
    
    let red = parse_color("#ff0000").expect("Failed to parse red");
    assert_eq!(red.r, 255);
    assert_eq!(red.g, 0);
    assert_eq!(red.b, 0);
    
    // Test invalid colors
    assert!(parse_color("invalid").is_err());
    assert!(parse_color("#gg0000").is_err());
    assert!(parse_color("#ff00").is_err());
}

#[test]
fn test_config_save_and_load() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.toml");
    
    // Create a custom config
    let mut config = Config::default();
    config.display.width = 1024;
    config.display.height = 768;
    config.font.size = 14.0;
    
    // Save to file
    let toml_str = toml::to_string(&config).expect("Failed to serialize config");
    fs::write(&config_path, toml_str).expect("Failed to write config");
    
    // Load from file
    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    let loaded_config: Config = toml::from_str(&content)
        .expect("Failed to deserialize config");
    
    assert_eq!(loaded_config.display.width, 1024);
    assert_eq!(loaded_config.display.height, 768);
    assert_eq!(loaded_config.font.size, 14.0);
}