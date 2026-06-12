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

use chrono::{Datelike, Local, Timelike};

#[derive(Default)]
pub struct Rtc7242x {
    mode12h: bool,
}

impl Rtc7242x {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self, reg: u8) -> u8 {
        let now = Local::now();
        let second = now.second();
        let minute = now.minute();
        let hour = now.hour();
        let day = now.day();
        let month = now.month();
        let year = now.year() as u32;

        let value = match reg & 0x0F {
            0 => second % 10,
            1 => (second / 10) & 0x07,
            2 => minute % 10,
            3 => (minute / 10) & 0x07,
            4 => hour % 10,
            5 => {
                if self.mode12h {
                    let mut v = ((hour % 12) / 10) & 0x01;
                    if hour >= 12 {
                        v |= 0x04;
                    }
                    v
                } else {
                    (hour / 10) & 0x03
                }
            }
            6 => day % 10,
            7 => (day / 10) & 0x03,
            8 => month % 10,
            9 => (month / 10) & 0x01,
            10 => year % 10,
            11 => (year / 10) % 10,
            12 => now.weekday().num_days_from_sunday(),
            15 => {
                if self.mode12h {
                    0
                } else {
                    0x04
                }
            }
            _ => 0,
        };
        value as u8
    }

    pub fn write(&mut self, reg: u8, value: u8) {
        if reg & 0x0F == 0x0F {
            self.mode12h = (value & 0x04) == 0;
        }
    }
}
