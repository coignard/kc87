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

use crate::core::machine::FRAME_TIME_US;
use serde::{Deserialize, Serialize};

pub const KBD_MAX_COLUMNS: usize = 12;
pub const KBD_MAX_LINES: usize = 12;
pub const KBD_MAX_MOD_KEYS: usize = 4;
pub const KBD_MAX_KEYS: usize = 256;
pub const KBD_MAX_PRESSED_KEYS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[rustfmt::skip]
pub enum Key {
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, Colon, Semicolon, Comma, Equals, Period, Question, At, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Caret, Underscore, Exclaim, Quote, Hash, Dollar, Percent, Ampersand, Apostrophe, ParenLeft, ParenRight, Asterisk, Plus, Less, Minus, Greater, Slash, LowerA, LowerB, LowerC, LowerD, LowerE, LowerF, LowerG, LowerH, LowerI, LowerJ, LowerK, LowerL, LowerM, LowerN, LowerO, LowerP, LowerQ, LowerR, LowerS, LowerT, LowerU, LowerV, LowerW, LowerX, LowerY, LowerZ, Stop, CursorLeft, CursorRight, CursorDown, CursorUp, Enter, Pause, Color, Home, Insert, Escape, List, Run, Space
}

impl Key {
    pub const ALL: [Key; 100] = [
        Key::Num0,
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
        Key::Colon,
        Key::Semicolon,
        Key::Comma,
        Key::Equals,
        Key::Period,
        Key::Question,
        Key::At,
        Key::A,
        Key::B,
        Key::C,
        Key::D,
        Key::E,
        Key::F,
        Key::G,
        Key::H,
        Key::I,
        Key::J,
        Key::K,
        Key::L,
        Key::M,
        Key::N,
        Key::O,
        Key::P,
        Key::Q,
        Key::R,
        Key::S,
        Key::T,
        Key::U,
        Key::V,
        Key::W,
        Key::X,
        Key::Y,
        Key::Z,
        Key::Caret,
        Key::Underscore,
        Key::Exclaim,
        Key::Quote,
        Key::Hash,
        Key::Dollar,
        Key::Percent,
        Key::Ampersand,
        Key::Apostrophe,
        Key::ParenLeft,
        Key::ParenRight,
        Key::Asterisk,
        Key::Plus,
        Key::Less,
        Key::Minus,
        Key::Greater,
        Key::Slash,
        Key::LowerA,
        Key::LowerB,
        Key::LowerC,
        Key::LowerD,
        Key::LowerE,
        Key::LowerF,
        Key::LowerG,
        Key::LowerH,
        Key::LowerI,
        Key::LowerJ,
        Key::LowerK,
        Key::LowerL,
        Key::LowerM,
        Key::LowerN,
        Key::LowerO,
        Key::LowerP,
        Key::LowerQ,
        Key::LowerR,
        Key::LowerS,
        Key::LowerT,
        Key::LowerU,
        Key::LowerV,
        Key::LowerW,
        Key::LowerX,
        Key::LowerY,
        Key::LowerZ,
        Key::Stop,
        Key::CursorLeft,
        Key::CursorRight,
        Key::CursorDown,
        Key::CursorUp,
        Key::Enter,
        Key::Pause,
        Key::Color,
        Key::Home,
        Key::Insert,
        Key::Escape,
        Key::List,
        Key::Run,
        Key::Space,
    ];

    pub const fn coords(self) -> (usize, usize, u32) {
        match self {
            Self::Num0 => (0, 0, 0),
            Self::Num1 => (1, 0, 0),
            Self::Num2 => (2, 0, 0),
            Self::Num3 => (3, 0, 0),
            Self::Num4 => (4, 0, 0),
            Self::Num5 => (5, 0, 0),
            Self::Num6 => (6, 0, 0),
            Self::Num7 => (7, 0, 0),
            Self::Num8 => (0, 1, 0),
            Self::Num9 => (1, 1, 0),
            Self::Colon => (2, 1, 0),
            Self::Semicolon => (3, 1, 0),
            Self::Comma => (4, 1, 0),
            Self::Equals => (5, 1, 0),
            Self::Period => (6, 1, 0),
            Self::Question => (7, 1, 0),
            Self::At => (0, 2, 0),
            Self::A => (1, 2, 0),
            Self::B => (2, 2, 0),
            Self::C => (3, 2, 0),
            Self::D => (4, 2, 0),
            Self::E => (5, 2, 0),
            Self::F => (6, 2, 0),
            Self::G => (7, 2, 0),
            Self::H => (0, 3, 0),
            Self::I => (1, 3, 0),
            Self::J => (2, 3, 0),
            Self::K => (3, 3, 0),
            Self::L => (4, 3, 0),
            Self::M => (5, 3, 0),
            Self::N => (6, 3, 0),
            Self::O => (7, 3, 0),
            Self::P => (0, 4, 0),
            Self::Q => (1, 4, 0),
            Self::R => (2, 4, 0),
            Self::S => (3, 4, 0),
            Self::T => (4, 4, 0),
            Self::U => (5, 4, 0),
            Self::V => (6, 4, 0),
            Self::W => (7, 4, 0),
            Self::X => (0, 5, 0),
            Self::Y => (1, 5, 0),
            Self::Z => (2, 5, 0),
            Self::Caret => (6, 5, 0),
            Self::Underscore => (0, 0, 1),
            Self::Exclaim => (1, 0, 1),
            Self::Quote => (2, 0, 1),
            Self::Hash => (3, 0, 1),
            Self::Dollar => (4, 0, 1),
            Self::Percent => (5, 0, 1),
            Self::Ampersand => (6, 0, 1),
            Self::Apostrophe => (7, 0, 1),
            Self::ParenLeft => (0, 1, 1),
            Self::ParenRight => (1, 1, 1),
            Self::Asterisk => (2, 1, 1),
            Self::Plus => (3, 1, 1),
            Self::Less => (4, 1, 1),
            Self::Minus => (5, 1, 1),
            Self::Greater => (6, 1, 1),
            Self::Slash => (7, 1, 1),
            Self::LowerA => (1, 2, 1),
            Self::LowerB => (2, 2, 1),
            Self::LowerC => (3, 2, 1),
            Self::LowerD => (4, 2, 1),
            Self::LowerE => (5, 2, 1),
            Self::LowerF => (6, 2, 1),
            Self::LowerG => (7, 2, 1),
            Self::LowerH => (0, 3, 1),
            Self::LowerI => (1, 3, 1),
            Self::LowerJ => (2, 3, 1),
            Self::LowerK => (3, 3, 1),
            Self::LowerL => (4, 3, 1),
            Self::LowerM => (5, 3, 1),
            Self::LowerN => (6, 3, 1),
            Self::LowerO => (7, 3, 1),
            Self::LowerP => (0, 4, 1),
            Self::LowerQ => (1, 4, 1),
            Self::LowerR => (2, 4, 1),
            Self::LowerS => (3, 4, 1),
            Self::LowerT => (4, 4, 1),
            Self::LowerU => (5, 4, 1),
            Self::LowerV => (6, 4, 1),
            Self::LowerW => (7, 4, 1),
            Self::LowerX => (0, 5, 1),
            Self::LowerY => (1, 5, 1),
            Self::LowerZ => (2, 5, 1),
            Self::Stop => (6, 6, 0),
            Self::CursorLeft => (0, 6, 0),
            Self::CursorRight => (1, 6, 0),
            Self::CursorDown => (2, 6, 0),
            Self::CursorUp => (3, 6, 0),
            Self::Enter => (5, 6, 0),
            Self::Pause => (4, 5, 0),
            Self::Color => (1, 7, 0),
            Self::Home => (3, 5, 0),
            Self::Insert => (5, 5, 0),
            Self::Escape => (4, 6, 0),
            Self::List => (4, 7, 0),
            Self::Run => (5, 7, 0),
            Self::Space => (7, 6, 0),
        }
    }

    pub const fn code(self) -> i32 {
        match self {
            Self::Num0 => 0x30,
            Self::Num1 => 0x31,
            Self::Num2 => 0x32,
            Self::Num3 => 0x33,
            Self::Num4 => 0x34,
            Self::Num5 => 0x35,
            Self::Num6 => 0x36,
            Self::Num7 => 0x37,
            Self::Num8 => 0x38,
            Self::Num9 => 0x39,
            Self::Colon => 0x3A,
            Self::Semicolon => 0x3B,
            Self::Comma => 0x2C,
            Self::Equals => 0x3D,
            Self::Period => 0x2E,
            Self::Question => 0x3F,
            Self::At => 0x40,
            Self::A => 0x41,
            Self::B => 0x42,
            Self::C => 0x43,
            Self::D => 0x44,
            Self::E => 0x45,
            Self::F => 0x46,
            Self::G => 0x47,
            Self::H => 0x48,
            Self::I => 0x49,
            Self::J => 0x4A,
            Self::K => 0x4B,
            Self::L => 0x4C,
            Self::M => 0x4D,
            Self::N => 0x4E,
            Self::O => 0x4F,
            Self::P => 0x50,
            Self::Q => 0x51,
            Self::R => 0x52,
            Self::S => 0x53,
            Self::T => 0x54,
            Self::U => 0x55,
            Self::V => 0x56,
            Self::W => 0x57,
            Self::X => 0x58,
            Self::Y => 0x59,
            Self::Z => 0x5A,
            Self::Caret => 0x5E,
            Self::Underscore => 0x5F,
            Self::Exclaim => 0x21,
            Self::Quote => 0x22,
            Self::Hash => 0x23,
            Self::Dollar => 0x24,
            Self::Percent => 0x25,
            Self::Ampersand => 0x26,
            Self::Apostrophe => 0x27,
            Self::ParenLeft => 0x28,
            Self::ParenRight => 0x29,
            Self::Asterisk => 0x2A,
            Self::Plus => 0x2B,
            Self::Less => 0x3C,
            Self::Minus => 0x2D,
            Self::Greater => 0x3E,
            Self::Slash => 0x2F,
            Self::LowerA => 0x61,
            Self::LowerB => 0x62,
            Self::LowerC => 0x63,
            Self::LowerD => 0x64,
            Self::LowerE => 0x65,
            Self::LowerF => 0x66,
            Self::LowerG => 0x67,
            Self::LowerH => 0x68,
            Self::LowerI => 0x69,
            Self::LowerJ => 0x6A,
            Self::LowerK => 0x6B,
            Self::LowerL => 0x6C,
            Self::LowerM => 0x6D,
            Self::LowerN => 0x6E,
            Self::LowerO => 0x6F,
            Self::LowerP => 0x70,
            Self::LowerQ => 0x71,
            Self::LowerR => 0x72,
            Self::LowerS => 0x73,
            Self::LowerT => 0x74,
            Self::LowerU => 0x75,
            Self::LowerV => 0x76,
            Self::LowerW => 0x77,
            Self::LowerX => 0x78,
            Self::LowerY => 0x79,
            Self::LowerZ => 0x7A,
            Self::Stop => 0x03,
            Self::CursorLeft => 0x08,
            Self::CursorRight => 0x09,
            Self::CursorDown => 0x0A,
            Self::CursorUp => 0x0B,
            Self::Enter => 0x0D,
            Self::Pause => 0x13,
            Self::Color => 0x14,
            Self::Home => 0x19,
            Self::Insert => 0x1A,
            Self::Escape => 0x1B,
            Self::List => 0x1C,
            Self::Run => 0x1D,
            Self::Space => 0x20,
        }
    }
}

#[derive(Clone, Copy, Default, Serialize)]
pub struct KeyState {
    pub key: i32,
    pub mask: u32,
    pub pressed_time: u64,
    pub released: bool,
}

#[derive(Clone, Serialize)]
pub struct Keyboard {
    pub cur_time: u64,
    pub sticky_time: u32,
    pub active_columns: u16,
    pub active_lines: u16,

    #[serde(skip)]
    pub key_masks: [u32; KBD_MAX_KEYS],
    #[serde(skip)]
    pub mod_masks: [u32; KBD_MAX_MOD_KEYS],

    pub key_buffer: [KeyState; KBD_MAX_PRESSED_KEYS],
    pub scanout_column_masks: [u16; KBD_MAX_LINES],
    pub scanout_line_masks: [u16; KBD_MAX_COLUMNS],

    pub cur_column_mask: u16,
    pub cur_scanout_line_mask: u16,
    pub cur_line_mask: u16,
    pub cur_scanout_column_mask: u16,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new(3)
    }
}

impl Keyboard {
    pub fn new(sticky_frames: u32) -> Self {
        Self {
            cur_time: 0,
            sticky_time: sticky_frames * FRAME_TIME_US,
            active_columns: 0,
            active_lines: 0,
            key_masks: [0; KBD_MAX_KEYS],
            mod_masks: [0; KBD_MAX_MOD_KEYS],
            key_buffer: [KeyState::default(); KBD_MAX_PRESSED_KEYS],
            scanout_column_masks: [0; KBD_MAX_LINES],
            scanout_line_masks: [0; KBD_MAX_COLUMNS],
            cur_column_mask: 0,
            cur_scanout_line_mask: 0,
            cur_line_mask: 0,
            cur_scanout_column_mask: 0,
        }
    }

    pub fn register_modifier(&mut self, layer: usize, column: usize, line: usize) {
        self.mod_masks[layer] = (1 << (layer + KBD_MAX_COLUMNS + KBD_MAX_LINES))
            | (1 << (column + KBD_MAX_LINES))
            | (1 << line);
    }

    pub fn register_modifier_line(&mut self, layer: usize, line: usize) {
        self.mod_masks[layer] = (1 << (layer + KBD_MAX_COLUMNS + KBD_MAX_LINES)) | (1 << line);
    }

    pub fn register_modifier_column(&mut self, layer: usize, column: usize) {
        self.mod_masks[layer] =
            (1 << (layer + KBD_MAX_COLUMNS + KBD_MAX_LINES)) | (1 << (column + KBD_MAX_LINES));
    }

    pub fn register_key(&mut self, key: i32, column: usize, line: usize, mod_mask: u32) {
        self.key_masks[key as usize] = (mod_mask << (KBD_MAX_COLUMNS + KBD_MAX_LINES))
            | (1 << (column + KBD_MAX_LINES))
            | (1 << line);
    }

    pub fn register_default_layout(&mut self) {
        const SHIFT_MODIFIER_LAYER: usize = 0;
        const SHIFT_KEY_COLUMN: usize = 0;
        const SHIFT_KEY_LINE: usize = 7;
        self.register_modifier(SHIFT_MODIFIER_LAYER, SHIFT_KEY_COLUMN, SHIFT_KEY_LINE);
        for key in Key::ALL {
            let (column, line, mod_mask) = key.coords();
            self.register_key(key.code(), column, line, mod_mask);
        }
    }

    #[inline(always)]
    fn columns(key_mask: u32) -> u16 {
        ((key_mask >> KBD_MAX_LINES) & ((1 << KBD_MAX_COLUMNS) - 1)) as u16
    }

    #[inline(always)]
    fn lines(key_mask: u32) -> u16 {
        (key_mask & ((1 << KBD_MAX_LINES) - 1)) as u16
    }

    #[inline(always)]
    fn mods(key_mask: u32) -> u32 {
        key_mask & (((1 << KBD_MAX_MOD_KEYS) - 1) << (KBD_MAX_COLUMNS + KBD_MAX_LINES))
    }

    fn test_lines_internal(&self, column_mask: u16) -> u16 {
        let mut line_bits = 0;
        for key in &self.key_buffer {
            let key_mask = key.mask;
            if key_mask != 0 {
                let key_col_mask = Self::columns(key_mask);
                if (key_col_mask & column_mask) == key_col_mask {
                    line_bits |= Self::lines(key_mask);
                }
                let key_mod_mask = Self::mods(key_mask);
                if key_mod_mask != 0 {
                    for mod_mask in &self.mod_masks {
                        if (mod_mask & key_mod_mask) != 0 {
                            let mod_col_mask = Self::columns(*mod_mask);
                            if mod_col_mask != 0 {
                                if (mod_col_mask & column_mask) == mod_col_mask {
                                    line_bits |= Self::lines(*mod_mask);
                                }
                            } else {
                                line_bits |= Self::lines(*mod_mask);
                            }
                        }
                    }
                }
            }
        }
        line_bits
    }

    fn test_columns_internal(&self, line_mask: u16) -> u16 {
        let mut column_bits = 0;
        for key in &self.key_buffer {
            let key_mask = key.mask;
            if key_mask != 0 {
                let key_line_mask = Self::lines(key_mask);
                if (key_line_mask & line_mask) == key_line_mask {
                    column_bits |= Self::columns(key_mask);
                }
                let key_mod_mask = Self::mods(key_mask);
                if key_mod_mask != 0 {
                    for mod_mask in &self.mod_masks {
                        if (mod_mask & key_mod_mask) != 0 {
                            let mod_line_mask = Self::lines(*mod_mask);
                            if mod_line_mask != 0 {
                                if (mod_line_mask & line_mask) == mod_line_mask {
                                    column_bits |= Self::columns(*mod_mask);
                                }
                            } else {
                                column_bits |= Self::columns(*mod_mask);
                            }
                        }
                    }
                }
            }
        }
        column_bits
    }

    fn update_scanout_masks(&mut self) {
        for line in 0..KBD_MAX_LINES {
            self.scanout_column_masks[line] = self.test_columns_internal(1 << line);
        }
        for col in 0..KBD_MAX_COLUMNS {
            self.scanout_line_masks[col] = self.test_lines_internal(1 << col);
        }
        self.cur_column_mask = 0;
        self.cur_scanout_line_mask = 0;
        self.cur_line_mask = 0;
        self.cur_scanout_column_mask = 0;
    }

    pub fn update(&mut self, frame_time_us: u32) {
        for k in &mut self.key_buffer {
            if k.released
                && (self.cur_time < k.pressed_time
                    || self.cur_time > (k.pressed_time + self.sticky_time as u64))
            {
                k.mask = 0;
                k.key = 0;
                k.pressed_time = 0;
                k.released = false;
            }
        }
        self.cur_time = self.cur_time.wrapping_add(frame_time_us as u64);
        self.update_scanout_masks();
    }

    pub fn key_down(&mut self, key: Key) {
        let key = key.code();
        for k in &mut self.key_buffer {
            if k.key == key {
                k.pressed_time = self.cur_time;
                self.update_scanout_masks();
                return;
            }
        }
        for k in &mut self.key_buffer {
            if k.mask == 0 {
                k.key = key;
                k.mask = self.key_masks[key as usize];
                k.pressed_time = self.cur_time;
                k.released = false;
                self.update_scanout_masks();
                return;
            }
        }
    }

    pub fn key_up(&mut self, key: Key) {
        let key = key.code();
        for k in &mut self.key_buffer {
            if k.key == key {
                k.released = true;
            }
        }
        self.update_scanout_masks();
    }

    pub fn test_lines(&mut self, column_mask: u16) -> u16 {
        if column_mask != self.cur_column_mask {
            let mut m = 0;
            for col in 0..KBD_MAX_COLUMNS {
                if (column_mask & (1 << col)) != 0 {
                    m |= self.scanout_line_masks[col];
                }
            }
            self.cur_scanout_line_mask = m;
            self.cur_column_mask = column_mask;
        }
        self.cur_scanout_line_mask
    }

    pub fn test_columns(&mut self, line_mask: u16) -> u16 {
        if line_mask != self.cur_line_mask {
            let mut m = 0;
            for line in 0..KBD_MAX_LINES {
                if (line_mask & (1 << line)) != 0 {
                    m |= self.scanout_column_masks[line];
                }
            }
            self.cur_scanout_column_mask = m;
            self.cur_line_mask = line_mask;
        }
        self.cur_scanout_column_mask
    }

    #[inline]
    pub fn set_active_columns(&mut self, column_mask: u16) {
        self.active_columns = column_mask;
    }

    #[inline]
    pub fn scan_lines(&mut self) -> u16 {
        self.test_lines(self.active_columns)
    }

    #[inline]
    pub fn set_active_lines(&mut self, line_mask: u16) {
        self.active_lines = line_mask;
    }

    #[inline]
    pub fn scan_columns(&mut self) -> u16 {
        self.test_columns(self.active_lines)
    }
}
