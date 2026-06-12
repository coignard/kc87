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

use crate::core::bus::Bus;
use crate::core::machine::MachineType;

pub const SCREEN_HEIGHT: usize = 192;
pub const COLUMNS: usize = 40;
pub const ROWS: usize = 24;

pub const BORDER_X: usize = 32;
pub const BORDER_Y: usize = 24;

pub const BASE_WIDTH: usize = COLUMNS * 8;
pub const FB_HEIGHT: usize = SCREEN_HEIGHT + 2 * BORDER_Y;

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
            frame_buffer: vec![0; fb_width * FB_HEIGHT * 4],
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
            self.frame_buffer = vec![0; self.fb_width * FB_HEIGHT * 4];
        }

        let video_ram = &bus.ram[0xEC00..0xF000];
        let color_ram = &bus.ram[0xE800..0xEC00];
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
        let frame_border = PALETTE[(border_color & 0x07) as usize];

        let gap_color = if has_color { frame_border } else { PALETTE[0] };

        for px in self.frame_buffer.chunks_exact_mut(4) {
            px.copy_from_slice(&frame_border);
        }

        if bus.graph_robotron_active() {
            let fg = PALETTE[(bus.graph_fg & 0x07) as usize];
            let bg = PALETTE[(bus.graph_bg & 0x07) as usize];
            let side = if bus.graph_border {
                frame_border
            } else {
                PALETTE[0]
            };
            for ay in 0..SCREEN_HEIGHT {
                let fb_y = BORDER_Y + ay;
                for ax in 0..BASE_WIDTH {
                    let color = if (32..32 + 256).contains(&ax) {
                        let gx = ax - 32;
                        let byte = ram_pixel[ay * 32 + gx / 8];
                        if (byte & (0x80 >> (gx % 8))) != 0 {
                            fg
                        } else {
                            bg
                        }
                    } else {
                        side
                    };
                    let base_idx = (fb_y * fb_width + BORDER_X + ax * hscale) * 4;
                    for s in 0..hscale {
                        let idx = base_idx + s * 4;
                        self.frame_buffer[idx..idx + 4].copy_from_slice(&color);
                    }
                }
            }
            return;
        }

        let (rows, row_height) = if mode_20_rows {
            (20usize, 9usize)
        } else {
            (ROWS, 8usize)
        };
        let columns = if want_wide { COLUMNS * 2 } else { COLUMNS };

        for ay in 0..SCREEN_HEIGHT {
            let fb_y = BORDER_Y + ay;
            let row = ay / row_height;
            let glyph_line = ay % row_height;

            if glyph_line < 8 && row < rows {
                for col in 0..columns {
                    let (vram, cram, bank_col) = if want_wide && (col & 1) != 0 {
                        (video_ram2, color_ram2, col / 2)
                    } else if want_wide {
                        (video_ram, color_ram, col / 2)
                    } else {
                        (video_ram, color_ram, col)
                    };
                    let offset = row * COLUMNS + bank_col;
                    let char_code = vram[offset] as usize;

                    let pixels = if krt_active {
                        ram_pixel[glyph_line * 0x400 + offset]
                    } else if chargen_active && char_code >= 0x80 {
                        chargen_ram[(char_code - 0x80) * 8 + glyph_line]
                    } else {
                        self.font_rom[(char_code << 3) | glyph_line]
                    };

                    let (bg, fg) = if !has_color {
                        (PALETTE[0], PALETTE[7])
                    } else {
                        let mut color_attr = cram[offset];
                        if (color_attr & 0x80) != 0 && (blink_flip_flop & 0x80) != 0 {
                            color_attr = ((color_attr & 0x07) << 4) | ((color_attr >> 4) & 0x07);
                        }
                        (
                            PALETTE[(color_attr & 0x07) as usize],
                            PALETTE[((color_attr >> 4) & 0x07) as usize],
                        )
                    };

                    let base_idx = (fb_y * fb_width + BORDER_X + col * 8) * 4;
                    for b in 0..8 {
                        let color = if (pixels & (0x80 >> b)) != 0 { fg } else { bg };
                        let idx = base_idx + b * 4;
                        self.frame_buffer[idx..idx + 4].copy_from_slice(&color);
                    }
                }
            } else if !has_color {
                let base_idx = (fb_y * fb_width + BORDER_X) * 4;
                for x in 0..content_width {
                    let idx = base_idx + x * 4;
                    self.frame_buffer[idx..idx + 4].copy_from_slice(&gap_color);
                }
            }
        }
    }
}
