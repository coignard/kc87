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

pub mod rtc7242x;
pub mod u855;
pub mod u857;

use u880::pins;

pub(crate) const INT_NEEDED: u8 = 1 << 0;
const INT_REQUESTED: u8 = 1 << 1;
const INT_SERVICED: u8 = 1 << 2;

#[inline]
#[must_use]
pub(crate) fn daisy_chain_step(mut current_pins: u64, state: &mut u8, vector: u8) -> u64 {
    if (current_pins & pins::RETI) != 0 && (*state & INT_SERVICED) != 0 {
        *state &= !INT_SERVICED;
        current_pins &= !pins::RETI;
    }

    if *state != 0 && (current_pins & pins::IEIO) != 0 {
        current_pins &= !pins::IEIO;

        if (*state & INT_NEEDED) != 0 {
            current_pins |= pins::INT;
            *state = (*state & !INT_NEEDED) | INT_REQUESTED;
        }

        if (*state & INT_REQUESTED) != 0
            && (current_pins & (pins::IORQ | pins::M1)) == (pins::IORQ | pins::M1)
        {
            current_pins = pins::set_data(current_pins, vector);
            *state = (*state & !INT_REQUESTED) | INT_SERVICED;
            current_pins &= !pins::INT;
        }
    }

    current_pins
}
