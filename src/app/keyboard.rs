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
use winit::keyboard::{Key as WinitKey, KeyCode, PhysicalKey};

use kc87::core::peripherals::keyboard::Key;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum KeyboardLayout {
    #[default]
    Smart,
    Qwertz,
}

pub struct KeyboardTranslator {
    layout: KeyboardLayout,
    shift_pressed: bool,
    held: HashMap<KeyCode, Key>,
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

    pub fn process_key(&mut self, event: &KeyEvent) -> Vec<(Key, bool)> {
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

    pub fn release_all(&mut self) -> Vec<(Key, bool)> {
        self.shift_pressed = false;
        self.held.drain().map(|(_, k)| (k, false)).collect()
    }

    fn resolve(&self, event: &KeyEvent, code: KeyCode) -> Option<Key> {
        if matches!(self.layout, KeyboardLayout::Smart)
            && let WinitKey::Character(ref ch) = event.logical_key
            && let Some(k) = map_smart_char(ch.as_str())
        {
            return Some(k);
        }
        map_keycode(code, self.shift_pressed)
    }
}

fn map_keycode(code: KeyCode, shift: bool) -> Option<Key> {
    match code {
        KeyCode::Digit0 => Some(if shift { Key::Underscore } else { Key::Num0 }),
        KeyCode::Digit1 => Some(if shift { Key::Exclaim } else { Key::Num1 }),
        KeyCode::Digit2 => Some(if shift { Key::Quote } else { Key::Num2 }),
        KeyCode::Digit3 => Some(if shift { Key::Hash } else { Key::Num3 }),
        KeyCode::Digit4 => Some(if shift { Key::Dollar } else { Key::Num4 }),
        KeyCode::Digit5 => Some(if shift { Key::Percent } else { Key::Num5 }),
        KeyCode::Digit6 => Some(if shift { Key::Ampersand } else { Key::Num6 }),
        KeyCode::Digit7 => Some(if shift { Key::Apostrophe } else { Key::Num7 }),
        KeyCode::Digit8 => Some(if shift { Key::ParenLeft } else { Key::Num8 }),
        KeyCode::Digit9 => Some(if shift { Key::ParenRight } else { Key::Num9 }),

        KeyCode::Semicolon => Some(if shift { Key::Less } else { Key::Semicolon }),
        KeyCode::Equal => Some(if shift { Key::Greater } else { Key::Equals }),
        KeyCode::Period => Some(if shift { Key::Slash } else { Key::Period }),
        KeyCode::Slash => Some(if shift { Key::Space } else { Key::Question }),
        KeyCode::Comma => Some(if shift { Key::Plus } else { Key::Comma }),
        KeyCode::Quote => Some(if shift { Key::Asterisk } else { Key::At }),
        KeyCode::Backslash => Some(if shift { Key::Space } else { Key::Colon }),
        KeyCode::Minus => Some(Key::Minus),

        KeyCode::KeyA => Some(if shift { Key::LowerA } else { Key::A }),
        KeyCode::KeyB => Some(if shift { Key::LowerB } else { Key::B }),
        KeyCode::KeyC => Some(if shift { Key::LowerC } else { Key::C }),
        KeyCode::KeyD => Some(if shift { Key::LowerD } else { Key::D }),
        KeyCode::KeyE => Some(if shift { Key::LowerE } else { Key::E }),
        KeyCode::KeyF => Some(if shift { Key::LowerF } else { Key::F }),
        KeyCode::KeyG => Some(if shift { Key::LowerG } else { Key::G }),
        KeyCode::KeyH => Some(if shift { Key::LowerH } else { Key::H }),
        KeyCode::KeyI => Some(if shift { Key::LowerI } else { Key::I }),
        KeyCode::KeyJ => Some(if shift { Key::LowerJ } else { Key::J }),
        KeyCode::KeyK => Some(if shift { Key::LowerK } else { Key::K }),
        KeyCode::KeyL => Some(if shift { Key::LowerL } else { Key::L }),
        KeyCode::KeyM => Some(if shift { Key::LowerM } else { Key::M }),
        KeyCode::KeyN => Some(if shift { Key::LowerN } else { Key::N }),
        KeyCode::KeyO => Some(if shift { Key::LowerO } else { Key::O }),
        KeyCode::KeyP => Some(if shift { Key::LowerP } else { Key::P }),
        KeyCode::KeyQ => Some(if shift { Key::LowerQ } else { Key::Q }),
        KeyCode::KeyR => Some(if shift { Key::LowerR } else { Key::R }),
        KeyCode::KeyS => Some(if shift { Key::LowerS } else { Key::S }),
        KeyCode::KeyT => Some(if shift { Key::LowerT } else { Key::T }),
        KeyCode::KeyU => Some(if shift { Key::LowerU } else { Key::U }),
        KeyCode::KeyV => Some(if shift { Key::LowerV } else { Key::V }),
        KeyCode::KeyW => Some(if shift { Key::LowerW } else { Key::W }),
        KeyCode::KeyX => Some(if shift { Key::LowerX } else { Key::X }),
        KeyCode::KeyY => Some(if shift { Key::LowerZ } else { Key::Z }),
        KeyCode::KeyZ => Some(if shift { Key::LowerY } else { Key::Y }),

        KeyCode::Backquote => Some(if shift { Key::Space } else { Key::Caret }),

        KeyCode::ArrowLeft => Some(Key::CursorLeft),
        KeyCode::ArrowRight => Some(Key::CursorRight),
        KeyCode::ArrowUp => Some(Key::CursorUp),
        KeyCode::ArrowDown => Some(Key::CursorDown),

        KeyCode::Backspace => Some(Key::CursorLeft),
        KeyCode::Escape => Some(if shift { Key::Escape } else { Key::Stop }),
        KeyCode::Enter | KeyCode::NumpadEnter => Some(Key::Enter),
        KeyCode::End => Some(Key::Stop),
        KeyCode::Space => Some(Key::Space),
        KeyCode::Insert => Some(Key::Insert),
        KeyCode::Home => Some(Key::Home),
        KeyCode::Pause => Some(Key::Pause),

        KeyCode::F1 => Some(Key::Color),
        KeyCode::F2 => Some(Key::List),
        KeyCode::F3 => Some(Key::Run),

        _ => None,
    }
}

fn map_smart_char(s: &str) -> Option<Key> {
    let mut chars = s.chars();
    let c = match (chars.next(), chars.next()) {
        (Some(c), None) => c,
        _ => return None,
    };
    match c {
        'a'..='z' => key_for_char(c.to_ascii_uppercase()),
        'A'..='Z' => key_for_char(c.to_ascii_lowercase()),
        ' ' | '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-'
        | '.' | '/' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | ':' | ';'
        | '<' | '=' | '>' | '?' | '@' | '^' | '_' => key_for_char(c),
        _ => None,
    }
}

fn key_for_char(c: char) -> Option<Key> {
    Some(match c {
        ' ' => Key::Space,
        'A' => Key::A,
        '&' => Key::Ampersand,
        '\'' => Key::Apostrophe,
        '*' => Key::Asterisk,
        '@' => Key::At,
        'B' => Key::B,
        'C' => Key::C,
        '^' => Key::Caret,
        ':' => Key::Colon,
        ',' => Key::Comma,
        'D' => Key::D,
        '$' => Key::Dollar,
        'E' => Key::E,
        '=' => Key::Equals,
        '!' => Key::Exclaim,
        'F' => Key::F,
        'G' => Key::G,
        '>' => Key::Greater,
        'H' => Key::H,
        '#' => Key::Hash,
        'I' => Key::I,
        'J' => Key::J,
        'K' => Key::K,
        'L' => Key::L,
        '<' => Key::Less,
        'a' => Key::LowerA,
        'b' => Key::LowerB,
        'c' => Key::LowerC,
        'd' => Key::LowerD,
        'e' => Key::LowerE,
        'f' => Key::LowerF,
        'g' => Key::LowerG,
        'h' => Key::LowerH,
        'i' => Key::LowerI,
        'j' => Key::LowerJ,
        'k' => Key::LowerK,
        'l' => Key::LowerL,
        'm' => Key::LowerM,
        'n' => Key::LowerN,
        'o' => Key::LowerO,
        'p' => Key::LowerP,
        'q' => Key::LowerQ,
        'r' => Key::LowerR,
        's' => Key::LowerS,
        't' => Key::LowerT,
        'u' => Key::LowerU,
        'v' => Key::LowerV,
        'w' => Key::LowerW,
        'x' => Key::LowerX,
        'y' => Key::LowerY,
        'z' => Key::LowerZ,
        'M' => Key::M,
        '-' => Key::Minus,
        'N' => Key::N,
        '0' => Key::Num0,
        '1' => Key::Num1,
        '2' => Key::Num2,
        '3' => Key::Num3,
        '4' => Key::Num4,
        '5' => Key::Num5,
        '6' => Key::Num6,
        '7' => Key::Num7,
        '8' => Key::Num8,
        '9' => Key::Num9,
        'O' => Key::O,
        'P' => Key::P,
        '(' => Key::ParenLeft,
        ')' => Key::ParenRight,
        '%' => Key::Percent,
        '.' => Key::Period,
        '+' => Key::Plus,
        'Q' => Key::Q,
        '?' => Key::Question,
        '"' => Key::Quote,
        'R' => Key::R,
        'S' => Key::S,
        ';' => Key::Semicolon,
        '/' => Key::Slash,
        'T' => Key::T,
        'U' => Key::U,
        '_' => Key::Underscore,
        'V' => Key::V,
        'W' => Key::W,
        'X' => Key::X,
        'Y' => Key::Y,
        'Z' => Key::Z,
        _ => return None,
    })
}
