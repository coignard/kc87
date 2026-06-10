// This file is part of kc87.
//
// Copyright (c) 2026  René Coignard <contact@renecoignard.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, KeyCode, PhysicalKey};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum KeyboardLayout {
    #[default]
    Smart,
    Qwertz,
}

pub struct KeyboardTranslator {
    layout: KeyboardLayout,
    shift_pressed: bool,
    held: HashMap<KeyCode, i32>,
}

impl Default for KeyboardTranslator {
    fn default() -> Self {
        Self::new(KeyboardLayout::default())
    }
}

impl KeyboardTranslator {
    pub fn new(layout: KeyboardLayout) -> Self {
        Self {
            layout,
            shift_pressed: false,
            held: HashMap::new(),
        }
    }

    pub fn process_key(&mut self, event: &KeyEvent) -> Vec<(i32, bool)> {
        let is_pressed = event.state == ElementState::Pressed;
        let mut actions = Vec::new();

        if let PhysicalKey::Code(code) = event.physical_key {
            if matches!(code, KeyCode::ShiftLeft | KeyCode::ShiftRight) {
                self.shift_pressed = is_pressed;
                return actions;
            }

            if is_pressed {
                let mapped_key = match self.held.get(&code) {
                    Some(&k) => Some(k),
                    None => {
                        let resolved = self.resolve(event, code);
                        if let Some(k) = resolved {
                            self.held.insert(code, k);
                        }
                        resolved
                    }
                };
                if let Some(k) = mapped_key {
                    actions.push((k, true));
                }
            } else if let Some(k) = self.held.remove(&code) {
                actions.push((k, false));
            }
        }

        actions
    }

    pub fn release_all(&mut self) -> Vec<(i32, bool)> {
        self.shift_pressed = false;
        self.held.drain().map(|(_, k)| (k, false)).collect()
    }

    fn resolve(&self, event: &KeyEvent, code: KeyCode) -> Option<i32> {
        if matches!(self.layout, KeyboardLayout::Smart)
            && let Key::Character(ref ch) = event.logical_key
            && let Some(k) = map_smart_char(ch.as_str())
        {
            return Some(k);
        }
        map_keycode(code, self.shift_pressed)
    }
}

fn map_smart_char(s: &str) -> Option<i32> {
    let mut chars = s.chars();
    let c = match (chars.next(), chars.next()) {
        (Some(c), None) => c,
        _ => return None,
    };
    match c {
        '0'..='9' => Some(c as i32),
        'a'..='z' => Some(c.to_ascii_uppercase() as i32),
        'A'..='Z' => Some(c.to_ascii_lowercase() as i32),
        ' ' | '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-'
        | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | '^' | '_' => Some(c as i32),
        _ => None,
    }
}

fn map_keycode(code: KeyCode, shift: bool) -> Option<i32> {
    match code {
        KeyCode::Digit0 => Some(if shift { b'_' as i32 } else { b'0' as i32 }),
        KeyCode::Digit1 => Some(if shift { b'!' as i32 } else { b'1' as i32 }),
        KeyCode::Digit2 => Some(if shift { b'"' as i32 } else { b'2' as i32 }),
        KeyCode::Digit3 => Some(if shift { b'#' as i32 } else { b'3' as i32 }),
        KeyCode::Digit4 => Some(if shift { b'$' as i32 } else { b'4' as i32 }),
        KeyCode::Digit5 => Some(if shift { b'%' as i32 } else { b'5' as i32 }),
        KeyCode::Digit6 => Some(if shift { b'&' as i32 } else { b'6' as i32 }),
        KeyCode::Digit7 => Some(if shift { b'\'' as i32 } else { b'7' as i32 }),
        KeyCode::Digit8 => Some(if shift { b'(' as i32 } else { b'8' as i32 }),
        KeyCode::Digit9 => Some(if shift { b')' as i32 } else { b'9' as i32 }),

        KeyCode::Semicolon => Some(if shift { b'<' as i32 } else { b';' as i32 }),
        KeyCode::Equal => Some(if shift { b'>' as i32 } else { b'=' as i32 }),
        KeyCode::Period => Some(if shift { b'/' as i32 } else { b'.' as i32 }),
        KeyCode::Slash => Some(if shift { b' ' as i32 } else { b'?' as i32 }),
        KeyCode::Minus => Some(b'-' as i32),
        KeyCode::Comma => Some(if shift { b'+' as i32 } else { b',' as i32 }),
        KeyCode::Quote => Some(if shift { b'*' as i32 } else { b'@' as i32 }),
        KeyCode::Backslash => Some(if shift { b' ' as i32 } else { b':' as i32 }),

        KeyCode::KeyA => Some(if shift { b'a' as i32 } else { b'A' as i32 }),
        KeyCode::KeyB => Some(if shift { b'b' as i32 } else { b'B' as i32 }),
        KeyCode::KeyC => Some(if shift { b'c' as i32 } else { b'C' as i32 }),
        KeyCode::KeyD => Some(if shift { b'd' as i32 } else { b'D' as i32 }),
        KeyCode::KeyE => Some(if shift { b'e' as i32 } else { b'E' as i32 }),
        KeyCode::KeyF => Some(if shift { b'f' as i32 } else { b'F' as i32 }),
        KeyCode::KeyG => Some(if shift { b'g' as i32 } else { b'G' as i32 }),
        KeyCode::KeyH => Some(if shift { b'h' as i32 } else { b'H' as i32 }),
        KeyCode::KeyI => Some(if shift { b'i' as i32 } else { b'I' as i32 }),
        KeyCode::KeyJ => Some(if shift { b'j' as i32 } else { b'J' as i32 }),
        KeyCode::KeyK => Some(if shift { b'k' as i32 } else { b'K' as i32 }),
        KeyCode::KeyL => Some(if shift { b'l' as i32 } else { b'L' as i32 }),
        KeyCode::KeyM => Some(if shift { b'm' as i32 } else { b'M' as i32 }),
        KeyCode::KeyN => Some(if shift { b'n' as i32 } else { b'N' as i32 }),
        KeyCode::KeyO => Some(if shift { b'o' as i32 } else { b'O' as i32 }),
        KeyCode::KeyP => Some(if shift { b'p' as i32 } else { b'P' as i32 }),
        KeyCode::KeyQ => Some(if shift { b'q' as i32 } else { b'Q' as i32 }),
        KeyCode::KeyR => Some(if shift { b'r' as i32 } else { b'R' as i32 }),
        KeyCode::KeyS => Some(if shift { b's' as i32 } else { b'S' as i32 }),
        KeyCode::KeyT => Some(if shift { b't' as i32 } else { b'T' as i32 }),
        KeyCode::KeyU => Some(if shift { b'u' as i32 } else { b'U' as i32 }),
        KeyCode::KeyV => Some(if shift { b'v' as i32 } else { b'V' as i32 }),
        KeyCode::KeyW => Some(if shift { b'w' as i32 } else { b'W' as i32 }),
        KeyCode::KeyX => Some(if shift { b'x' as i32 } else { b'X' as i32 }),
        KeyCode::KeyY => Some(if shift { b'z' as i32 } else { b'Z' as i32 }),
        KeyCode::KeyZ => Some(if shift { b'y' as i32 } else { b'Y' as i32 }),

        KeyCode::Backquote => Some(if shift { b' ' as i32 } else { b'^' as i32 }),

        KeyCode::ArrowLeft => Some(0x08),
        KeyCode::ArrowRight => Some(0x09),
        KeyCode::ArrowUp => Some(0x0A),
        KeyCode::ArrowDown => Some(0x0B),

        KeyCode::Backspace => Some(0x08),
        KeyCode::Escape => Some(if shift { 0x1B } else { 0x03 }),
        KeyCode::Enter | KeyCode::NumpadEnter => Some(0x0D),
        KeyCode::End => Some(0x03),
        KeyCode::Space => Some(0x20),
        KeyCode::Insert => Some(0x1A),
        KeyCode::Home => Some(0x19),
        KeyCode::Pause => Some(0x13),

        KeyCode::F1 => Some(0x14),
        KeyCode::F2 => Some(0x1C),
        KeyCode::F3 => Some(0x1D),

        _ => None,
    }
}
