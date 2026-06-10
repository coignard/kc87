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

const MAX_MIDI_BUFFER: usize = 256;

#[derive(Serialize)]
pub struct MidiInterface {
    pub(crate) out_buffer: Vec<(u8, u64)>,
}

impl Default for MidiInterface {
    fn default() -> Self {
        Self {
            out_buffer: Vec::with_capacity(MAX_MIDI_BUFFER),
        }
    }
}

impl MidiInterface {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn push_byte(&mut self, byte: u8, cycle_count: u64) {
        if self.out_buffer.len() < MAX_MIDI_BUFFER {
            self.out_buffer.push((byte, cycle_count));
        }
    }
}
