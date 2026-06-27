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

use crate::core::bus::{Bus, memory_map};
use crate::core::machine::MachineType;

pub const SCREEN_HEIGHT: usize = 192;
pub const COLUMNS: usize = 40;
pub const ROWS: usize = 24;

pub const BORDER_X: usize = 32;
pub const BORDER_Y: usize = 24;

pub const GLYPH_WIDTH: usize = 8;
pub const GLYPH_HEIGHT: usize = 8;
pub const BASE_WIDTH: usize = COLUMNS * GLYPH_WIDTH;
pub const FB_HEIGHT: usize = SCREEN_HEIGHT + 2 * BORDER_Y;

const BYTES_PER_PIXEL: usize = 4;
const WIDE_SCALE: usize = 2;
const ROWS_20_MODE: usize = 20;
const ROW_HEIGHT_20: usize = 9;
const ROW_HEIGHT_24: usize = 8;
const ALT_FONT_BASE: usize = 0x80;
const BANK_STRIDE: usize = 0x400;

const COLOR_INDEX_MASK: u8 = 0x07;
const COLOR_NIBBLE_SHIFT: u8 = 4;
const BLINK_ATTR_BIT: u8 = 0x80;
const GLYPH_MSB: u8 = 0x80;

const GRAPH_X_OFFSET: usize = 32;
const GRAPH_WIDTH: usize = 256;
const GRAPH_BYTES_PER_ROW: usize = 32;

const PALETTE: [[u8; 4]; 8] = [
    [0x00, 0x00, 0x00, 0xFF],
    [0xFF, 0x00, 0x00, 0xFF],
    [0x00, 0xFF, 0x00, 0xFF],
    [0xFF, 0xFF, 0x00, 0xFF],
    [0x00, 0x00, 0xFF, 0xFF],
    [0xFF, 0x00, 0xFF, 0xFF],
    [0x00, 0xFF, 0xFF, 0xFF],
    [0xFF, 0xFF, 0xFF, 0xFF],
];

const MONO_BLACK: usize = 0;
const MONO_WHITE: usize = 7;

fn resolve_color(index: u8, has_color: bool) -> [u8; 4] {
    let index = (index & COLOR_INDEX_MASK) as usize;
    if has_color {
        PALETTE[index]
    } else if index == MONO_BLACK {
        PALETTE[MONO_BLACK]
    } else {
        PALETTE[MONO_WHITE]
    }
}

pub struct VideoRenderer {
    pub font_rom: Vec<u8>,
    frame_buffer: Vec<u8>,
    c80_enabled: bool,
    content_width: usize,
    fb_width: usize,
}

impl VideoRenderer {
    pub fn new(font_rom: Vec<u8>, c80_enabled: bool) -> Self {
        let content_width = BASE_WIDTH;
        let fb_width = content_width + 2 * BORDER_X;
        Self {
            font_rom,
            frame_buffer: vec![0; fb_width * FB_HEIGHT * BYTES_PER_PIXEL],
            c80_enabled,
            content_width,
            fb_width,
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.fb_width as u32
    }

    #[inline]
    pub fn height(&self) -> u32 {
        FB_HEIGHT as u32
    }

    #[inline]
    pub fn frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }

    pub fn render_frame(&mut self, bus: &Bus, machine_type: MachineType) {
        let want_wide = self.c80_enabled && bus.c80_active;
        let target_width = if want_wide {
            BASE_WIDTH * 2
        } else {
            BASE_WIDTH
        };
        if target_width != self.content_width {
            self.content_width = target_width;
            self.fb_width = target_width + 2 * BORDER_X;
            self.frame_buffer = vec![0; self.fb_width * FB_HEIGHT * BYTES_PER_PIXEL];
        }

        let video_ram =
            &bus.ram[memory_map::VIDEO_RAM_START as usize..memory_map::VIDEO_RAM_END as usize];
        let color_ram =
            &bus.ram[memory_map::COLOR_RAM_START as usize..memory_map::COLOR_RAM_END as usize];
        let video_ram2 = &bus.ram_video2[..];
        let color_ram2 = &bus.ram_color2[..];
        let blink_flip_flop = bus.blink_flip_flop;
        let border_color = bus.border_color();
        let mode_20_rows = bus.mode_20_rows();
        let chargen_ram = &bus.chargen_ram[..];
        let chargen_active = bus.chargen_active;
        let ram_pixel = &bus.ram_pixel[..];
        let krt_active = bus.graph_krt_active();

        let content_width = self.content_width;
        let fb_width = self.fb_width;
        let hscale = content_width / BASE_WIDTH;

        let has_color = machine_type == MachineType::KC87;
        let frame_border = resolve_color(border_color, has_color);

        for px in self.frame_buffer.chunks_exact_mut(BYTES_PER_PIXEL) {
            px.copy_from_slice(&frame_border);
        }

        if bus.graph_robotron_active() {
            let fg = PALETTE[(bus.graph_fg & COLOR_INDEX_MASK) as usize];
            let bg = PALETTE[(bus.graph_bg & COLOR_INDEX_MASK) as usize];
            let side = if bus.graph_border {
                frame_border
            } else {
                PALETTE[0]
            };
            for ay in 0..SCREEN_HEIGHT {
                let fb_y = BORDER_Y + ay;
                for ax in 0..BASE_WIDTH {
                    let color = if (GRAPH_X_OFFSET..GRAPH_X_OFFSET + GRAPH_WIDTH).contains(&ax) {
                        let gx = ax - GRAPH_X_OFFSET;
                        let byte = ram_pixel[ay * GRAPH_BYTES_PER_ROW + gx / GLYPH_WIDTH];
                        if (byte & (GLYPH_MSB >> (gx % GLYPH_WIDTH))) != 0 {
                            fg
                        } else {
                            bg
                        }
                    } else {
                        side
                    };
                    let base_idx = (fb_y * fb_width + BORDER_X + ax * hscale) * BYTES_PER_PIXEL;
                    for s in 0..hscale {
                        let idx = base_idx + s * BYTES_PER_PIXEL;
                        self.frame_buffer[idx..idx + BYTES_PER_PIXEL].copy_from_slice(&color);
                    }
                }
            }
            return;
        }

        let (rows, row_height) = if mode_20_rows {
            (ROWS_20_MODE, ROW_HEIGHT_20)
        } else {
            (ROWS, ROW_HEIGHT_24)
        };
        let columns = if want_wide {
            COLUMNS * WIDE_SCALE
        } else {
            COLUMNS
        };

        for ay in 0..SCREEN_HEIGHT {
            let fb_y = BORDER_Y + ay;
            let row = ay / row_height;
            let glyph_line = ay % row_height;

            if glyph_line < GLYPH_HEIGHT && row < rows {
                for col in 0..columns {
                    let (vram, cram, bank_col) = if want_wide && (col & 1) != 0 {
                        (video_ram2, color_ram2, col / WIDE_SCALE)
                    } else if want_wide {
                        (video_ram, color_ram, col / WIDE_SCALE)
                    } else {
                        (video_ram, color_ram, col)
                    };
                    let offset = row * COLUMNS + bank_col;
                    let char_code = vram[offset] as usize;

                    let pixels = if krt_active {
                        ram_pixel[glyph_line * BANK_STRIDE + offset]
                    } else if chargen_active && char_code >= ALT_FONT_BASE {
                        chargen_ram[(char_code - ALT_FONT_BASE) * GLYPH_HEIGHT + glyph_line]
                    } else {
                        self.font_rom[char_code * GLYPH_HEIGHT + glyph_line]
                    };

                    let (bg, fg) = if !has_color {
                        (PALETTE[0], PALETTE[7])
                    } else {
                        let mut color_attr = cram[offset];
                        if (color_attr & BLINK_ATTR_BIT) != 0
                            && (blink_flip_flop & BLINK_ATTR_BIT) != 0
                        {
                            color_attr = ((color_attr & COLOR_INDEX_MASK) << COLOR_NIBBLE_SHIFT)
                                | ((color_attr >> COLOR_NIBBLE_SHIFT) & COLOR_INDEX_MASK);
                        }
                        (
                            PALETTE[(color_attr & COLOR_INDEX_MASK) as usize],
                            PALETTE
                                [((color_attr >> COLOR_NIBBLE_SHIFT) & COLOR_INDEX_MASK) as usize],
                        )
                    };

                    let base_idx =
                        (fb_y * fb_width + BORDER_X + col * GLYPH_WIDTH) * BYTES_PER_PIXEL;
                    for b in 0..GLYPH_WIDTH {
                        let color = if (pixels & (GLYPH_MSB >> b)) != 0 {
                            fg
                        } else {
                            bg
                        };
                        let idx = base_idx + b * BYTES_PER_PIXEL;
                        self.frame_buffer[idx..idx + BYTES_PER_PIXEL].copy_from_slice(&color);
                    }
                }
            }
        }
    }
}
