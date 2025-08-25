use anyhow::Result;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum KeyCode {
    Char(char),
    Enter,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    F(u8),
    Escape,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Modifiers: u8 {
        const SHIFT = 0b00000001;
        const CTRL = 0b00000010;
        const ALT = 0b00000100;
        const SUPER = 0b00001000;
    }
}

impl Key {
    pub fn new(code: KeyCode, modifiers: Modifiers) -> Self {
        Self { code, modifiers }
    }
    
    #[allow(dead_code)]
    pub fn char(c: char) -> Self {
        Self::new(KeyCode::Char(c), Modifiers::empty())
    }
    
    #[allow(dead_code)]
    pub fn ctrl(c: char) -> Self {
        Self::new(KeyCode::Char(c), Modifiers::CTRL)
    }
    
    #[allow(dead_code)]
    pub fn alt(c: char) -> Self {
        Self::new(KeyCode::Char(c), Modifiers::ALT)
    }
    
    #[allow(dead_code)]
    pub fn shift(c: char) -> Self {
        Self::new(KeyCode::Char(c), Modifiers::SHIFT)
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        match (&self.code, &self.modifiers) {
            (KeyCode::Char(c), modifiers) => {
                if modifiers.contains(Modifiers::CTRL) {
                    match c.to_ascii_lowercase() {
                        'a'..='z' => vec![(*c as u8) - b'a' + 1],
                        '@' => vec![0],
                        '[' => vec![27],
                        '\\' => vec![28],
                        ']' => vec![29],
                        '^' => vec![30],
                        '_' => vec![31],
                        '?' => vec![127],
                        _ => c.to_string().into_bytes(),
                    }
                } else if modifiers.contains(Modifiers::ALT) {
                    let mut bytes = vec![27]; // ESC
                    bytes.extend(c.to_string().into_bytes());
                    bytes
                } else {
                    c.to_string().into_bytes()
                }
            }
            (KeyCode::Enter, _) => vec![b'\r'],
            (KeyCode::Tab, _) => vec![b'\t'],
            (KeyCode::Backspace, _) => vec![127],
            (KeyCode::Delete, _) => b"\x1b[3~".to_vec(),
            (KeyCode::Insert, _) => b"\x1b[2~".to_vec(),
            (KeyCode::Home, _) => {
                if self.modifiers.contains(Modifiers::CTRL) {
                    b"\x1b[1;5H".to_vec()
                } else {
                    b"\x1b[H".to_vec()
                }
            }
            (KeyCode::End, _) => {
                if self.modifiers.contains(Modifiers::CTRL) {
                    b"\x1b[1;5F".to_vec()
                } else {
                    b"\x1b[F".to_vec()
                }
            }
            (KeyCode::PageUp, _) => b"\x1b[5~".to_vec(),
            (KeyCode::PageDown, _) => b"\x1b[6~".to_vec(),
            (KeyCode::Up, _) => {
                if self.modifiers.contains(Modifiers::CTRL) {
                    b"\x1b[1;5A".to_vec()
                } else if self.modifiers.contains(Modifiers::SHIFT) {
                    b"\x1b[1;2A".to_vec()
                } else {
                    b"\x1b[A".to_vec()
                }
            }
            (KeyCode::Down, _) => {
                if self.modifiers.contains(Modifiers::CTRL) {
                    b"\x1b[1;5B".to_vec()
                } else if self.modifiers.contains(Modifiers::SHIFT) {
                    b"\x1b[1;2B".to_vec()
                } else {
                    b"\x1b[B".to_vec()
                }
            }
            (KeyCode::Left, _) => {
                if self.modifiers.contains(Modifiers::CTRL) {
                    b"\x1b[1;5D".to_vec()
                } else if self.modifiers.contains(Modifiers::SHIFT) {
                    b"\x1b[1;2D".to_vec()
                } else {
                    b"\x1b[D".to_vec()
                }
            }
            (KeyCode::Right, _) => {
                if self.modifiers.contains(Modifiers::CTRL) {
                    b"\x1b[1;5C".to_vec()
                } else if self.modifiers.contains(Modifiers::SHIFT) {
                    b"\x1b[1;2C".to_vec()
                } else {
                    b"\x1b[C".to_vec()
                }
            }
            (KeyCode::F(n), _) => {
                match n {
                    1 => b"\x1bOP".to_vec(),
                    2 => b"\x1bOQ".to_vec(),
                    3 => b"\x1bOR".to_vec(),
                    4 => b"\x1bOS".to_vec(),
                    5 => b"\x1b[15~".to_vec(),
                    6 => b"\x1b[17~".to_vec(),
                    7 => b"\x1b[18~".to_vec(),
                    8 => b"\x1b[19~".to_vec(),
                    9 => b"\x1b[20~".to_vec(),
                    10 => b"\x1b[21~".to_vec(),
                    11 => b"\x1b[23~".to_vec(),
                    12 => b"\x1b[24~".to_vec(),
                    _ => vec![],
                }
            }
            (KeyCode::Escape, _) => vec![27],
            _ => vec![],
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        
        if self.modifiers.contains(Modifiers::CTRL) {
            parts.push("Ctrl");
        }
        if self.modifiers.contains(Modifiers::ALT) {
            parts.push("Alt");
        }
        if self.modifiers.contains(Modifiers::SHIFT) {
            parts.push("Shift");
        }
        if self.modifiers.contains(Modifiers::SUPER) {
            parts.push("Super");
        }
        
        let key_name = match &self.code {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Insert => "Insert".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Escape => "Escape".to_string(),
            KeyCode::CapsLock => "CapsLock".to_string(),
            KeyCode::ScrollLock => "ScrollLock".to_string(),
            KeyCode::NumLock => "NumLock".to_string(),
            KeyCode::PrintScreen => "PrintScreen".to_string(),
            KeyCode::Pause => "Pause".to_string(),
            KeyCode::Menu => "Menu".to_string(),
        };
        
        parts.push(&key_name);
        write!(f, "{}", parts.join("+"))
    }
}

#[allow(dead_code)]
pub fn parse_key_binding(s: &str) -> Result<Key> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = Modifiers::empty();
    let mut key_code = None;
    
    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" => modifiers.insert(Modifiers::CTRL),
            "alt" => modifiers.insert(Modifiers::ALT),
            "shift" => modifiers.insert(Modifiers::SHIFT),
            "super" | "cmd" => modifiers.insert(Modifiers::SUPER),
            "enter" => key_code = Some(KeyCode::Enter),
            "tab" => key_code = Some(KeyCode::Tab),
            "backspace" => key_code = Some(KeyCode::Backspace),
            "delete" => key_code = Some(KeyCode::Delete),
            "insert" => key_code = Some(KeyCode::Insert),
            "home" => key_code = Some(KeyCode::Home),
            "end" => key_code = Some(KeyCode::End),
            "pageup" => key_code = Some(KeyCode::PageUp),
            "pagedown" => key_code = Some(KeyCode::PageDown),
            "up" => key_code = Some(KeyCode::Up),
            "down" => key_code = Some(KeyCode::Down),
            "left" => key_code = Some(KeyCode::Left),
            "right" => key_code = Some(KeyCode::Right),
            "escape" => key_code = Some(KeyCode::Escape),
            s if s.starts_with('f') && s.len() > 1 => {
                if let Ok(n) = s[1..].parse::<u8>() {
                    if (1..=12).contains(&n) {
                        key_code = Some(KeyCode::F(n));
                    }
                }
            }
            s if s.len() == 1 => {
                if let Some(c) = s.chars().next() {
                    key_code = Some(KeyCode::Char(c));
                }
            }
            _ => return Err(anyhow::anyhow!("Unknown key: {}", part)),
        }
    }
    
    let code = key_code.ok_or_else(|| anyhow::anyhow!("No key code found in: {}", s))?;
    Ok(Key::new(code, modifiers))
}