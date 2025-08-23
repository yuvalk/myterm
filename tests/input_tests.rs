use myterm::input::{Key, KeyCode, Modifiers, parse_key_binding};

#[test]
fn test_key_creation() {
    let key = Key::char('a');
    assert_eq!(key.code, KeyCode::Char('a'));
    assert_eq!(key.modifiers, Modifiers::empty());
    
    let ctrl_c = Key::ctrl('c');
    assert_eq!(ctrl_c.code, KeyCode::Char('c'));
    assert!(ctrl_c.modifiers.contains(Modifiers::CTRL));
    
    let alt_f4 = Key::new(KeyCode::F(4), Modifiers::ALT);
    assert_eq!(alt_f4.code, KeyCode::F(4));
    assert!(alt_f4.modifiers.contains(Modifiers::ALT));
}

#[test]
fn test_key_to_bytes() {
    // Regular characters
    assert_eq!(Key::char('a').to_bytes(), b"a");
    assert_eq!(Key::char('A').to_bytes(), b"A");
    
    // Control characters
    assert_eq!(Key::ctrl('a').to_bytes(), vec![1]); // Ctrl+A = 0x01
    assert_eq!(Key::ctrl('c').to_bytes(), vec![3]); // Ctrl+C = 0x03
    assert_eq!(Key::ctrl('z').to_bytes(), vec![26]); // Ctrl+Z = 0x1A
    
    // Special keys
    assert_eq!(Key::new(KeyCode::Enter, Modifiers::empty()).to_bytes(), b"\r");
    assert_eq!(Key::new(KeyCode::Tab, Modifiers::empty()).to_bytes(), b"\t");
    assert_eq!(Key::new(KeyCode::Backspace, Modifiers::empty()).to_bytes(), vec![127]);
    
    // Arrow keys
    assert_eq!(Key::new(KeyCode::Up, Modifiers::empty()).to_bytes(), b"\x1b[A");
    assert_eq!(Key::new(KeyCode::Down, Modifiers::empty()).to_bytes(), b"\x1b[B");
    assert_eq!(Key::new(KeyCode::Right, Modifiers::empty()).to_bytes(), b"\x1b[C");
    assert_eq!(Key::new(KeyCode::Left, Modifiers::empty()).to_bytes(), b"\x1b[D");
    
    // Function keys
    assert_eq!(Key::new(KeyCode::F(1), Modifiers::empty()).to_bytes(), b"\x1bOP");
    assert_eq!(Key::new(KeyCode::F(2), Modifiers::empty()).to_bytes(), b"\x1bOQ");
    
    // Alt+character
    assert_eq!(Key::alt('a').to_bytes(), b"\x1ba");
    assert_eq!(Key::alt('x').to_bytes(), b"\x1bx");
}

#[test]
fn test_key_display() {
    assert_eq!(Key::char('a').to_string(), "a");
    assert_eq!(Key::ctrl('c').to_string(), "Ctrl+c");
    assert_eq!(Key::alt('f').to_string(), "Alt+f");
    assert_eq!(Key::new(KeyCode::F(1), Modifiers::SHIFT).to_string(), "Shift+F1");
    assert_eq!(Key::new(KeyCode::Enter, Modifiers::CTRL | Modifiers::ALT).to_string(), "Ctrl+Alt+Enter");
}

#[test]
fn test_parse_key_binding() {
    // Simple characters
    let key = parse_key_binding("a").expect("Failed to parse 'a'");
    assert_eq!(key.code, KeyCode::Char('a'));
    assert_eq!(key.modifiers, Modifiers::empty());
    
    // Ctrl combinations
    let ctrl_c = parse_key_binding("Ctrl+c").expect("Failed to parse 'Ctrl+c'");
    assert_eq!(ctrl_c.code, KeyCode::Char('c'));
    assert!(ctrl_c.modifiers.contains(Modifiers::CTRL));
    
    // Multiple modifiers
    let complex = parse_key_binding("Ctrl+Shift+F1").expect("Failed to parse 'Ctrl+Shift+F1'");
    assert_eq!(complex.code, KeyCode::F(1));
    assert!(complex.modifiers.contains(Modifiers::CTRL));
    assert!(complex.modifiers.contains(Modifiers::SHIFT));
    
    // Special keys
    let enter = parse_key_binding("Enter").expect("Failed to parse 'Enter'");
    assert_eq!(enter.code, KeyCode::Enter);
    
    let alt_tab = parse_key_binding("Alt+Tab").expect("Failed to parse 'Alt+Tab'");
    assert_eq!(alt_tab.code, KeyCode::Tab);
    assert!(alt_tab.modifiers.contains(Modifiers::ALT));
    
    // Invalid keys should fail
    assert!(parse_key_binding("Invalid+Key").is_err());
    assert!(parse_key_binding("Ctrl+").is_err());
}