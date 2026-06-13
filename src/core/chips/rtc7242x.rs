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

const REG_MASK: u8 = 0x0F;
const MODE_REG: u8 = 0x0F;

const SEC_UNITS: u8 = 0;
const SEC_TENS: u8 = 1;
const MIN_UNITS: u8 = 2;
const MIN_TENS: u8 = 3;
const HOUR_UNITS: u8 = 4;
const HOUR_TENS: u8 = 5;
const DAY_UNITS: u8 = 6;
const DAY_TENS: u8 = 7;
const MONTH_UNITS: u8 = 8;
const MONTH_TENS: u8 = 9;
const YEAR_UNITS: u8 = 10;
const YEAR_TENS: u8 = 11;
const WEEKDAY: u8 = 12;

const BCD_BASE: u32 = 10;
const TENS_3BIT_MASK: u32 = 0x07;
const TENS_2BIT_MASK: u32 = 0x03;
const TENS_1BIT_MASK: u32 = 0x01;
const HOUR12_TENS_MASK: u32 = 0x01;
const NOON_HOUR: u32 = 12;
const AMPM_24H_BIT: u8 = 0x04;

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

        let value = match reg & REG_MASK {
            SEC_UNITS => second % BCD_BASE,
            SEC_TENS => (second / BCD_BASE) & TENS_3BIT_MASK,
            MIN_UNITS => minute % BCD_BASE,
            MIN_TENS => (minute / BCD_BASE) & TENS_3BIT_MASK,
            HOUR_UNITS => hour % BCD_BASE,
            HOUR_TENS => {
                if self.mode12h {
                    let mut v = ((hour % NOON_HOUR) / BCD_BASE) & HOUR12_TENS_MASK;
                    if hour >= NOON_HOUR {
                        v |= AMPM_24H_BIT as u32;
                    }
                    v
                } else {
                    (hour / BCD_BASE) & TENS_2BIT_MASK
                }
            }
            DAY_UNITS => day % BCD_BASE,
            DAY_TENS => (day / BCD_BASE) & TENS_2BIT_MASK,
            MONTH_UNITS => month % BCD_BASE,
            MONTH_TENS => (month / BCD_BASE) & TENS_1BIT_MASK,
            YEAR_UNITS => year % BCD_BASE,
            YEAR_TENS => (year / BCD_BASE) % BCD_BASE,
            WEEKDAY => now.weekday().num_days_from_sunday(),
            MODE_REG => {
                if self.mode12h {
                    0
                } else {
                    AMPM_24H_BIT as u32
                }
            }
            _ => 0,
        };
        value as u8
    }

    pub fn write(&mut self, reg: u8, value: u8) {
        if reg & REG_MASK == MODE_REG {
            self.mode12h = (value & AMPM_24H_BIT) == 0;
        }
    }
}
