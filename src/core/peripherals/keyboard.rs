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

use serde::Serialize;

pub const KBD_MAX_COLUMNS: usize = 12;
pub const KBD_MAX_LINES: usize = 12;
pub const KBD_MAX_MOD_KEYS: usize = 4;
pub const KBD_MAX_KEYS: usize = 256;
pub const KBD_MAX_PRESSED_KEYS: usize = 4;

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
            sticky_time: sticky_frames * 16667,
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

    pub fn key_down(&mut self, key: i32) {
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

    pub fn key_up(&mut self, key: i32) {
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
