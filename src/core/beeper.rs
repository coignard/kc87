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

const FIXEDPOINT_SCALE: i32 = 16;
const DCADJ_BUFLEN: usize = 128;
const DCADJ_MASK: usize = DCADJ_BUFLEN - 1;

#[derive(Serialize, Clone)]
pub struct Beeper {
    state: u8,
    period: i32,
    counter: i32,
    base_volume: f32,
    volume: f32,
    pub sample: f32,

    #[serde(skip)]
    dcadj_sum: f32,
    #[serde(skip)]
    dcadj_pos: usize,
    #[serde(skip)]
    dcadj_buf: [f32; DCADJ_BUFLEN],
}

impl Beeper {
    pub fn new(tick_hz: u32, sound_hz: u32, base_volume: f32) -> Self {
        let period = ((tick_hz * FIXEDPOINT_SCALE as u32) / sound_hz) as i32;
        Self {
            state: 0,
            period,
            counter: period,
            base_volume,
            volume: 1.0,
            sample: 0.0,
            dcadj_sum: 0.0,
            dcadj_pos: 0,
            dcadj_buf: [0.0; DCADJ_BUFLEN],
        }
    }

    pub fn reset(&mut self) {
        self.state = 0;
        self.counter = self.period;
        self.sample = 0.0;
        self.dcadj_sum = 0.0;
        self.dcadj_pos = 0;
        self.dcadj_buf.fill(0.0);
    }

    #[inline(always)]
    pub fn toggle(&mut self) {
        self.state ^= 1;
    }

    #[inline(always)]
    fn dc_adjust(&mut self, s: f32) {
        self.dcadj_sum -= self.dcadj_buf[self.dcadj_pos];
        self.dcadj_sum += s;
        self.dcadj_buf[self.dcadj_pos] = s;
        self.dcadj_pos = (self.dcadj_pos + 1) & DCADJ_MASK;
    }

    #[inline(always)]
    pub fn tick(&mut self, enabled: bool) -> bool {
        let phase = if enabled { self.state } else { 1 };
        let current_val = (phase as f32) * self.volume * self.base_volume;
        self.dc_adjust(current_val);

        self.counter -= FIXEDPOINT_SCALE;
        if self.counter <= 0 {
            self.counter += self.period;
            self.sample = self.dcadj_sum / (DCADJ_BUFLEN as f32);
            true
        } else {
            false
        }
    }
}
