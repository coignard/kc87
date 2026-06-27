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

const DC_BLOCKER_ALPHA: f32 = 0.999;
const PHASE_HYSTERESIS: f32 = 0.02;

#[derive(Clone, Copy)]
pub struct TapeTract {
    dc: f32,
    prev_in: f32,
    phase: bool,
}

impl TapeTract {
    pub fn new() -> Self {
        Self {
            dc: 0.0,
            prev_in: 0.0,
            phase: false,
        }
    }

    pub fn push(&mut self, sample: f32) -> Option<bool> {
        self.dc = sample - self.prev_in + DC_BLOCKER_ALPHA * self.dc;
        self.prev_in = sample;

        let next = if self.dc > PHASE_HYSTERESIS {
            true
        } else if self.dc < -PHASE_HYSTERESIS {
            false
        } else {
            self.phase
        };

        if next != self.phase {
            self.phase = next;
            Some(next)
        } else {
            None
        }
    }
}

impl Default for TapeTract {
    fn default() -> Self {
        Self::new()
    }
}
