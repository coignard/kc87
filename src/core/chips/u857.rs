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

use serde::{Deserialize, Serialize};
use u880::pins;

pub const CE: u64 = 1 << 40;
pub const CS0: u64 = 1 << 41;
pub const CS1: u64 = 1 << 42;
pub const CLKTRG0: u64 = 1 << 43;
pub const CLKTRG1: u64 = 1 << 44;
pub const CLKTRG2: u64 = 1 << 45;
pub const CLKTRG3: u64 = 1 << 46;
pub const ZCTO0: u64 = 1 << 47;
pub const ZCTO1: u64 = 1 << 48;
pub const ZCTO2: u64 = 1 << 49;

const CTRL_EI: u8 = 1 << 7;
const CTRL_MODE: u8 = 1 << 6;
const CTRL_MODE_COUNTER: u8 = 1 << 6;
const CTRL_MODE_TIMER: u8 = 0;

const CTRL_PRESCALER: u8 = 1 << 5;
const CTRL_PRESCALER_16: u8 = 0;

const CTRL_EDGE: u8 = 1 << 4;
const CTRL_EDGE_RISING: u8 = 1 << 4;

const CTRL_TRIGGER: u8 = 1 << 3;
const CTRL_TRIGGER_WAIT: u8 = 1 << 3;

const CTRL_CONST_FOLLOWS: u8 = 1 << 2;
const CTRL_RESET: u8 = 1 << 1;
const CTRL_CONTROL: u8 = 1 << 0;

const INT_NEEDED: u8 = 1 << 0;
const INT_REQUESTED: u8 = 1 << 1;
const INT_SERVICED: u8 = 1 << 2;

const NUM_CHANNELS: usize = 4;

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
struct U857Channel {
    control: u8,
    constant: u8,
    down_counter: u8,
    prescaler: u8,
    int_vector: u8,
    trigger_edge: bool,
    waiting_for_trigger: bool,
    ext_trigger: bool,
    prescaler_mask: u8,
    int_state: u8,
    running: bool,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct U857 {
    chn: [U857Channel; NUM_CHANNELS],
    pub pins: u64,
}

impl Default for U857 {
    fn default() -> Self {
        Self::new()
    }
}

impl U857 {
    pub fn new() -> Self {
        let mut ctc = Self {
            chn: [U857Channel::default(); NUM_CHANNELS],
            pins: 0,
        };
        ctc.reset();
        ctc
    }

    pub fn reset(&mut self) {
        for ch in &mut self.chn {
            ch.control = CTRL_RESET;
            ch.constant = 0;
            ch.down_counter = 0;
            ch.waiting_for_trigger = false;
            ch.trigger_edge = false;
            ch.prescaler_mask = 0x0F;
            ch.int_state = 0;
            ch.running = false;
        }
    }

    fn counter_zero(&mut self, mut current_pins: u64, chn_id: usize) -> u64 {
        let ch = &mut self.chn[chn_id];
        if (ch.control & CTRL_EI) != 0 {
            ch.int_state |= INT_NEEDED;
        }

        if chn_id < 3 {
            current_pins |= ZCTO0 << chn_id;
            self.pins = current_pins;
        }

        ch.down_counter = ch.constant;
        current_pins
    }

    fn active_edge(&mut self, mut current_pins: u64, chn_id: usize) -> u64 {
        let ch = &mut self.chn[chn_id];
        if (ch.control & CTRL_MODE) == CTRL_MODE_COUNTER {
            ch.down_counter = ch.down_counter.wrapping_sub(1);
            if ch.down_counter == 0 {
                current_pins = self.counter_zero(current_pins, chn_id);
            }
        } else if ch.waiting_for_trigger {
            ch.waiting_for_trigger = false;
            ch.down_counter = ch.constant;
        }
        current_pins
    }

    fn write(&mut self, mut current_pins: u64, chn_id: usize, data: u8) -> u64 {
        if (self.chn[chn_id].control & CTRL_CONST_FOLLOWS) != 0 {
            let ch = &mut self.chn[chn_id];
            ch.control &= !(CTRL_CONST_FOLLOWS | CTRL_RESET);
            ch.constant = data;

            if (ch.control & CTRL_MODE) == CTRL_MODE_TIMER {
                if (ch.control & CTRL_TRIGGER) == CTRL_TRIGGER_WAIT {
                    ch.waiting_for_trigger = true;
                } else {
                    ch.down_counter = ch.constant;
                }
            } else {
                ch.down_counter = ch.constant;
            }
        } else if (data & CTRL_CONTROL) != 0 {
            let ch = &mut self.chn[chn_id];
            let old_ctrl = ch.control;
            ch.control = data;
            if (data & CTRL_RESET) != 0 {
                ch.running = false;
            }
            ch.trigger_edge = (data & CTRL_EDGE) == CTRL_EDGE_RISING;
            ch.prescaler_mask = if (ch.control & CTRL_PRESCALER) == CTRL_PRESCALER_16 {
                0x0F
            } else {
                0xFF
            };

            if (old_ctrl & CTRL_EDGE) != (ch.control & CTRL_EDGE) {
                current_pins = self.active_edge(current_pins, chn_id);
            }
        } else if chn_id == 0 {
            for i in 0..NUM_CHANNELS {
                self.chn[i].int_vector = (data & 0xF8) + (i as u8 * 2);
            }
        }
        current_pins
    }

    fn handle_iorq(&mut self, mut current_pins: u64) -> u64 {
        let chn_id = (((current_pins & (CS0 | CS1)) / CS0) & 3) as usize;

        if (current_pins & pins::RD) != 0 {
            let data = self.chn[chn_id].down_counter;
            current_pins = pins::set_data(current_pins, data);
        } else {
            let data = pins::data(current_pins);
            current_pins = self.write(current_pins, chn_id, data);
        }
        current_pins
    }

    fn tick_internal(&mut self, mut current_pins: u64) -> u64 {
        current_pins &= !(ZCTO0 | ZCTO1 | ZCTO2);

        for chn_id in 0..NUM_CHANNELS {
            let ch = &mut self.chn[chn_id];
            let clktrg_pin = CLKTRG0 << chn_id;

            if ch.waiting_for_trigger || (ch.control & CTRL_MODE) == CTRL_MODE_COUNTER {
                let trg = (current_pins & clktrg_pin) != 0;
                if trg != ch.ext_trigger {
                    ch.ext_trigger = trg;
                    if ch.trigger_edge == trg {
                        current_pins = self.active_edge(current_pins, chn_id);
                    }
                }
            } else if (ch.control & (CTRL_MODE | CTRL_RESET | CTRL_CONST_FOLLOWS))
                == CTRL_MODE_TIMER
            {
                if ch.running {
                    ch.prescaler = ch.prescaler.wrapping_sub(1);
                    if (ch.prescaler & ch.prescaler_mask) == 0 {
                        ch.down_counter = ch.down_counter.wrapping_sub(1);
                        if ch.down_counter == 0 {
                            current_pins = self.counter_zero(current_pins, chn_id);
                        }
                    }
                } else {
                    ch.running = true;
                }
            }
        }
        current_pins
    }

    fn handle_int(&mut self, mut current_pins: u64) -> u64 {
        for ch in &mut self.chn {
            if (current_pins & pins::RETI) != 0 && (ch.int_state & INT_SERVICED) != 0 {
                ch.int_state &= !INT_SERVICED;
                current_pins &= !pins::RETI;
            }

            if ch.int_state != 0 && (current_pins & pins::IEIO) != 0 {
                current_pins &= !pins::IEIO;

                if (ch.int_state & INT_NEEDED) != 0 {
                    current_pins |= pins::INT;
                    ch.int_state = (ch.int_state & !INT_NEEDED) | INT_REQUESTED;
                }

                if (ch.int_state & INT_REQUESTED) != 0
                    && (current_pins & (pins::IORQ | pins::M1)) == (pins::IORQ | pins::M1)
                {
                    current_pins = pins::set_data(current_pins, ch.int_vector);
                    ch.int_state = (ch.int_state & !INT_REQUESTED) | INT_SERVICED;
                    current_pins &= !pins::INT;
                }
            }
        }
        current_pins
    }

    pub fn tick(&mut self, mut current_pins: u64) -> u64 {
        if (current_pins & (CE | pins::IORQ | pins::M1)) == (CE | pins::IORQ) {
            current_pins = self.handle_iorq(current_pins);
        }
        current_pins = self.tick_internal(current_pins);
        current_pins = self.handle_int(current_pins);

        self.pins = current_pins;
        current_pins
    }
}
