use livesplit_core::hotkey::Hotkey;
use serde::Deserialize;
use std::str::FromStr;
use winit::event::ModifiersState;
use winit::event::VirtualKeyCode;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseKeyError;

#[derive(Debug, Hash, PartialEq, Eq, Deserialize, Copy, Clone)]
pub struct Key {
    pub key: VirtualKeyCode,
    pub mods: ModifiersState,
}

impl Key {
    pub fn new(key: VirtualKeyCode) -> Self {
        Self::from(key, ModifiersState::empty())
    }
    pub fn ctrl(key: VirtualKeyCode) -> Self {
        Self::from(key, ModifiersState::CTRL)
    }
    pub fn cmd(key: VirtualKeyCode) -> Self {
        Self::from(key, ModifiersState::LOGO)
    }
    pub fn shift(key: VirtualKeyCode) -> Self {
        Self::from(key, ModifiersState::SHIFT)
    }
    pub fn alt(key: VirtualKeyCode) -> Self {
        Self::from(key, ModifiersState::ALT)
    }
    pub fn from(key: VirtualKeyCode, mods: ModifiersState) -> Self {
        Self { key, mods }
    }
}
impl From<&Hotkey> for Key {
    fn from(s: &Hotkey) -> Self {
        (*s).into()
    }
}

impl From<Hotkey> for Key {
    fn from(s: Hotkey) -> Self {
        let mut mods = ModifiersState::empty();
        if s.modifiers
            .intersects(livesplit_core::hotkey::Modifiers::SHIFT)
        {
            mods |= ModifiersState::SHIFT;
        }
        if s.modifiers
            .intersects(livesplit_core::hotkey::Modifiers::CONTROL)
        {
            mods |= ModifiersState::CTRL;
        }
        if s.modifiers
            .intersects(livesplit_core::hotkey::Modifiers::ALT)
        {
            mods |= ModifiersState::ALT;
        }
        if s.modifiers
            .intersects(livesplit_core::hotkey::Modifiers::META)
        {
            mods |= ModifiersState::LOGO;
        }
        let name = s.key_code.name();
        let key = str_to_virtual_key_code(name).unwrap();
        Key::from(key, mods)
    }
}

fn str_to_mods(s: &str) -> Result<ModifiersState, ParseKeyError> {
    let mut modifiers = ModifiersState::empty();
    for modifier in s.split('+').map(str::trim) {
        match modifier {
            "Ctrl" => modifiers.insert(ModifiersState::CTRL),
            "Alt" => modifiers.insert(ModifiersState::ALT),
            "Meta" => modifiers.insert(ModifiersState::LOGO),
            "Shift" => modifiers.insert(ModifiersState::SHIFT),
            _ => return Err(ParseKeyError),
        }
    }
    Ok(modifiers)
}
impl FromStr for Key {
    type Err = ParseKeyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((modifiers, key_code)) = s.rsplit_once('+') {
            let mods = str_to_mods(modifiers.trim())?;
            let key = str_to_virtual_key_code(key_code.trim())?;
            Ok(Self { key, mods })
        } else {
            let key = str_to_virtual_key_code(s)?;
            Ok(Self {
                key,
                mods: ModifiersState::empty(),
            })
        }
    }
}

fn str_to_virtual_key_code(s: &str) -> Result<VirtualKeyCode, ParseKeyError> {
    let k = match s {
        "Digit0" | "0" => VirtualKeyCode::Key0,
        "Digit1" | "1" => VirtualKeyCode::Key1,
        "Digit2" | "2" => VirtualKeyCode::Key2,
        "Digit3" | "3" => VirtualKeyCode::Key3,
        "Digit4" | "4" => VirtualKeyCode::Key4,
        "Digit5" | "5" => VirtualKeyCode::Key5,
        "Digit6" | "6" => VirtualKeyCode::Key6,
        "Digit7" | "7" => VirtualKeyCode::Key7,
        "Digit8" | "8" => VirtualKeyCode::Key8,
        "Digit9" | "9" => VirtualKeyCode::Key9,
        "Numpad0" => VirtualKeyCode::Numpad0,
        "Numpad1" => VirtualKeyCode::Numpad1,
        "Numpad2" => VirtualKeyCode::Numpad2,
        "Numpad3" => VirtualKeyCode::Numpad3,
        "Numpad4" => VirtualKeyCode::Numpad4,
        "Numpad5" => VirtualKeyCode::Numpad5,
        "Numpad6" => VirtualKeyCode::Numpad6,
        "Numpad7" => VirtualKeyCode::Numpad7,
        "Numpad8" => VirtualKeyCode::Numpad8,
        "Numpad9" => VirtualKeyCode::Numpad9,
        "KeyA" | "A" => VirtualKeyCode::A,
        "KeyB" | "B" => VirtualKeyCode::B,
        "KeyC" | "C" => VirtualKeyCode::C,
        "KeyD" | "D" => VirtualKeyCode::D,
        "KeyE" | "E" => VirtualKeyCode::E,
        "KeyF" | "F" => VirtualKeyCode::F,
        "KeyG" | "G" => VirtualKeyCode::G,
        "KeyH" | "H" => VirtualKeyCode::H,
        "KeyI" | "I" => VirtualKeyCode::I,
        "KeyJ" | "J" => VirtualKeyCode::J,
        "KeyK" | "K" => VirtualKeyCode::K,
        "KeyL" | "L" => VirtualKeyCode::L,
        "KeyM" | "M" => VirtualKeyCode::M,
        "KeyN" | "N" => VirtualKeyCode::N,
        "KeyO" | "O" => VirtualKeyCode::O,
        "KeyP" | "P" => VirtualKeyCode::P,
        "KeyQ" | "Q" => VirtualKeyCode::Q,
        "KeyR" | "R" => VirtualKeyCode::R,
        "KeyS" | "S" => VirtualKeyCode::S,
        "KeyT" | "T" => VirtualKeyCode::T,
        "KeyU" | "U" => VirtualKeyCode::U,
        "KeyV" | "V" => VirtualKeyCode::V,
        "KeyW" | "W" => VirtualKeyCode::W,
        "KeyX" | "X" => VirtualKeyCode::X,
        "KeyY" | "Y" => VirtualKeyCode::Y,
        "KeyZ" | "Z" => VirtualKeyCode::Z,
        "Backslash" => VirtualKeyCode::Backslash,
        "BracketLeft" => VirtualKeyCode::LBracket,
        "BracketRight" => VirtualKeyCode::RBracket,
        "Comma" => VirtualKeyCode::Comma,
        "Minus" => VirtualKeyCode::Minus,
        "Period" => VirtualKeyCode::Period,
        "Quote" => VirtualKeyCode::Apostrophe,
        "Semicolon" => VirtualKeyCode::Semicolon,
        "Slash" => VirtualKeyCode::Slash,
        "AltLeft" => VirtualKeyCode::LAlt,
        "AltRight" => VirtualKeyCode::RAlt,
        "Backspace" => VirtualKeyCode::Back,
        "ControlLeft" => VirtualKeyCode::LControl,
        "ControlRight" => VirtualKeyCode::RControl,
        "Enter" => VirtualKeyCode::Return,
        "MetaLeft" => VirtualKeyCode::LWin,
        "MetaRight" => VirtualKeyCode::RWin,
        "ShiftLeft" => VirtualKeyCode::LShift,
        "ShiftRight" => VirtualKeyCode::RShift,
        "Space" => VirtualKeyCode::Space,
        "Tab" => VirtualKeyCode::Tab,
        "Convert" => VirtualKeyCode::Convert,
        "Delete" => VirtualKeyCode::Delete,
        "End" => VirtualKeyCode::End,
        "Home" => VirtualKeyCode::Home,
        "Insert" => VirtualKeyCode::Insert,
        "PageDown" => VirtualKeyCode::PageDown,
        "PageUp" => VirtualKeyCode::PageUp,
        "ArrowDown" => VirtualKeyCode::Down,
        "ArrowUp" => VirtualKeyCode::Up,
        "ArrowLeft" => VirtualKeyCode::Left,
        "ArrowRight" => VirtualKeyCode::Right,
        "NumLock" => VirtualKeyCode::Numlock,
        "NumpadAdd" => VirtualKeyCode::NumpadAdd,
        "NumpadComma" => VirtualKeyCode::NumpadComma,
        "NumpadDecimal" => VirtualKeyCode::NumpadDecimal,
        "NumpadDivide" => VirtualKeyCode::NumpadDivide,
        "NumpadEnter" => VirtualKeyCode::NumpadEnter,
        "NumpadEqual" => VirtualKeyCode::NumpadEquals,
        "NumpadMultiply" => VirtualKeyCode::NumpadMultiply,
        "NumpadSubstract" => VirtualKeyCode::NumpadSubtract,
        "Escape" => VirtualKeyCode::Escape,
        //"Backquote" => VirtualKeyCode::???,
        //"CapsLock" => VirtualKeyCode::,
        //"ContextMenu" => VirtualKeyCode::,
        //"Help" => VirtualKeyCode::Help,
        //"NumpadBackspace" => VirtualKeyCode::,
        //"NumpadClear" => VirtualKeyCode::,
        //"NumpadClearEntry" => VirtualKeyCode::,
        //"NumpadHash" => VirtualKeyCode::,
        //"NumparParenLeft" => VirtualKeyCode::,
        //"NumpadParenRight" => VirtualKeyCode::,
        //"NumpadStar" => VirtualKeyCode::,
        _ => {
            dbg!("unimplemented", s);
            return Err(ParseKeyError);
        }
    };
    Ok(k)
}
/*
impl From<Hotkey> for FBKeyCode {
    fn from(key: Hotkey) -> Self {
        ls_to_fb(key)
    }
}

pub fn is_ctrl(window: &Window) -> bool {
    window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl)
}
pub fn is_alt(window: &Window) -> bool {
    window.is_key_down(Key::LeftAlt) || window.is_key_down(Key::RightAlt)
}
pub fn is_shift(window: &Window) -> bool {
    window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift)
}
pub fn is_super(window: &Window) -> bool {
    window.is_key_down(Key::LeftSuper) || window.is_key_down(Key::RightSuper)
}

pub fn get_modifier(window: &Window) -> Modifiers {
    let mut m = Modifiers::empty();
    if is_ctrl(&window) {
        m |= Modifiers::CONTROL;
    }
    if is_alt(&window) {
        m |= Modifiers::ALT;
    }
    if is_super(&window) {
        m |= Modifiers::META;
    }
    if is_shift(&window) {
        m |= Modifiers::SHIFT;
    }
    m
}
*/
/*
fn ls_to_fb(key: Hotkey) -> FBKeyCode {
    let key_code = match key.key_code {
        KeyCode::Space => Key::Space,
        KeyCode::KeyA => Key::A,
        KeyCode::KeyB => Key::B,
        KeyCode::KeyC => Key::C,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyF => Key::F,
        KeyCode::KeyG => Key::G,
        KeyCode::KeyH => Key::H,
        KeyCode::KeyI => Key::I,
        KeyCode::KeyJ => Key::J,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyL => Key::L,
        KeyCode::KeyM => Key::M,
        KeyCode::KeyN => Key::N,
        KeyCode::KeyO => Key::O,
        KeyCode::KeyP => Key::P,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyT => Key::T,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyV => Key::V,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyX => Key::X,
        KeyCode::KeyY => Key::Y,
        KeyCode::KeyZ => Key::Z,
        KeyCode::Digit0 => Key::Key0,
        KeyCode::Digit1 => Key::Key1,
        KeyCode::Digit2 => Key::Key2,
        KeyCode::Digit3 => Key::Key3,
        KeyCode::Digit4 => Key::Key4,
        KeyCode::Digit5 => Key::Key5,
        KeyCode::Digit6 => Key::Key6,
        KeyCode::Digit7 => Key::Key7,
        KeyCode::Digit8 => Key::Key8,
        KeyCode::Digit9 => Key::Key9,

        KeyCode::ArrowDown => Key::Down,
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::ArrowUp => Key::Up,
        KeyCode::Quote => Key::Apostrophe,

        KeyCode::Backquote => Key::Backquote,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::Comma => Key::Comma,
        KeyCode::Equal => Key::Equal,
        KeyCode::BracketLeft => Key::LeftBracket,
        KeyCode::Minus => Key::Minus,
        KeyCode::Period => Key::Period,
        KeyCode::BracketRight => Key::RightBracket,
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Slash => Key::Slash,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Delete,
        KeyCode::End => Key::End,
        KeyCode::Enter => Key::Enter,
        KeyCode::Escape => Key::Escape,
        KeyCode::Home => Key::Home,
        KeyCode::Insert => Key::Insert,
        KeyCode::ContextMenu => Key::Menu,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::Pause => Key::Pause,
        KeyCode::Tab => Key::Tab,
        KeyCode::NumLock => Key::NumLock,
        KeyCode::CapsLock => Key::CapsLock,
        KeyCode::ScrollLock => Key::ScrollLock,
        KeyCode::ShiftLeft => Key::LeftShift,
        KeyCode::ShiftRight => Key::RightShift,
        KeyCode::ControlLeft => Key::LeftCtrl,
        KeyCode::ControlRight => Key::RightCtrl,
        KeyCode::Numpad0 => Key::NumPad0,
        KeyCode::Numpad1 => Key::NumPad1,
        KeyCode::Numpad2 => Key::NumPad2,
        KeyCode::Numpad3 => Key::NumPad3,
        KeyCode::Numpad4 => Key::NumPad4,
        KeyCode::Numpad5 => Key::NumPad5,
        KeyCode::Numpad6 => Key::NumPad6,
        KeyCode::Numpad7 => Key::NumPad7,
        KeyCode::Numpad8 => Key::NumPad8,
        KeyCode::Numpad9 => Key::NumPad9,
        KeyCode::NumpadDecimal => Key::NumPadDot,
        KeyCode::NumpadDivide => Key::NumPadSlash,
        KeyCode::NumpadMultiply => Key::NumPadAsterisk,
        KeyCode::NumpadSubtract => Key::NumPadMinus,
        KeyCode::NumpadAdd => Key::NumPadPlus,
        KeyCode::NumpadEnter => Key::NumPadEnter,
        KeyCode::AltLeft => Key::LeftAlt,
        KeyCode::AltRight => Key::RightAlt,
        KeyCode::MetaLeft => Key::LeftSuper,
        KeyCode::MetaRight => Key::RightSuper,
        _ => Key::Unknown,
    };
    FBKeyCode {
        key_code,
        modifiers: key.modifiers,
    }
}
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct FBKeyCode {
    pub key_code: Key,
    pub modifiers: Modifiers,
}
impl FBKeyCode {
    pub fn new(key: Key) -> Self {
        Self {
            key_code: key,
            modifiers: Modifiers::empty(),
        }
    }
    pub fn new_ctrl(key: Key) -> Self {
        let mut m = Self::new(key);
        m.modifiers |= Modifiers::CONTROL;
        m
    }
    pub fn new_meta(key: Key) -> Self {
        let mut m = Self::new(key);
        m.modifiers |= Modifiers::META;
        m
    }
}
*/
