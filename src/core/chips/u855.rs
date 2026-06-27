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

use super::{INT_NEEDED, daisy_chain_step};

pub const CE: u64 = 1 << 40;
pub const BASEL: u64 = 1 << 41;
pub const CDSEL: u64 = 1 << 42;
pub const ARDY: u64 = 1 << 43;
pub const BRDY: u64 = 1 << 44;
pub const ASTB: u64 = 1 << 45;
pub const BSTB: u64 = 1 << 46;

const PA_SHIFT: usize = 48;
const PB_SHIFT: usize = 56;

const PA_MASK: u64 = 0xFF << PA_SHIFT;
const PB_MASK: u64 = 0xFF << PB_SHIFT;

const MODE_OUTPUT: u8 = 0;
const MODE_INPUT: u8 = 1;
const MODE_BIDIRECTIONAL: u8 = 2;
const MODE_BITCONTROL: u8 = 3;

const INTCTRL_EI: u8 = 1 << 7;
const INTCTRL_MASK_FOLLOWS: u8 = 1 << 4;

const CTRL_CMD_MASK: u8 = 0x0F;
const CMD_VECTOR_BIT: u8 = 0x01;
const CMD_SET_MODE: u8 = 0x0F;
const CMD_INT_CONTROL: u8 = 0x07;
const CMD_INT_ENABLE: u8 = 0x03;
const MODE_SHIFT: u8 = 6;
const INTCTRL_BITS_MASK: u8 = 0xF0;
const PORTA_STATUS_MASK: u8 = 0xC0;
const PORTB_STATUS_SHIFT: u8 = 4;
const INT_MASK_ALL: u8 = 0xFF;
const PORT_FLOAT: u8 = 0xFF;
const INT_LOGIC_MASK: u8 = 0x60;
const INT_LOGIC_OR_LOW: u8 = 0x00;
const INT_LOGIC_OR_HIGH: u8 = 0x20;
const INT_LOGIC_AND_LOW: u8 = 0x40;
const INT_LOGIC_AND_HIGH: u8 = 0x60;

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
struct U855Port {
    input: u8,
    output: u8,
    mode: u8,
    io_select: u8,
    int_vector: u8,
    int_control: u8,
    int_mask: u8,
    int_state: u8,
    int_enabled: bool,
    expect_io_select: bool,
    expect_int_mask: bool,
    bctrl_match: bool,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct U855 {
    ports: [U855Port; 2],
    reset_active: bool,
    pub pins: u64,
}

impl Default for U855 {
    fn default() -> Self {
        Self::new()
    }
}

impl U855 {
    pub fn new() -> Self {
        let mut chip = Self {
            ports: [U855Port::default(); 2],
            reset_active: true,
            pins: 0,
        };
        chip.reset();
        chip
    }

    #[inline]
    pub fn output_a(&self) -> u8 {
        self.ports[0].output
    }

    pub fn reset(&mut self) {
        for port in &mut self.ports {
            port.mode = MODE_INPUT;
            port.output = 0;
            port.io_select = 0;
            port.int_control &= !INTCTRL_EI;
            port.int_mask = INT_MASK_ALL;
            port.int_enabled = false;
            port.expect_int_mask = false;
            port.expect_io_select = false;
            port.bctrl_match = false;
            port.int_state = 0;
        }
        self.reset_active = true;
    }

    fn write_ctrl(&mut self, port_id: usize, data: u8) {
        self.reset_active = false;
        let p = &mut self.ports[port_id];

        if p.expect_io_select {
            p.io_select = data;
            p.int_enabled = (p.int_control & INTCTRL_EI) != 0;
            p.expect_io_select = false;
        } else if p.expect_int_mask {
            p.int_mask = data;
            p.int_enabled = (p.int_control & INTCTRL_EI) != 0;
            p.expect_int_mask = false;
        } else {
            let ctrl = data & CTRL_CMD_MASK;
            if (ctrl & CMD_VECTOR_BIT) == 0 {
                p.int_vector = data;
                p.int_control |= INTCTRL_EI;
                p.int_enabled = true;
            } else if ctrl == CMD_SET_MODE {
                p.mode = data >> MODE_SHIFT;
                if p.mode == MODE_BITCONTROL {
                    p.expect_io_select = true;
                    p.int_enabled = false;
                    p.bctrl_match = false;
                }
            } else if ctrl == CMD_INT_CONTROL {
                p.int_control = data & INTCTRL_BITS_MASK;
                if (data & INTCTRL_MASK_FOLLOWS) != 0 {
                    p.expect_int_mask = true;
                    p.int_enabled = false;
                    p.int_state &= !INT_NEEDED;
                    p.bctrl_match = false;
                } else {
                    p.int_enabled = (p.int_control & INTCTRL_EI) != 0;
                }
            } else if ctrl == CMD_INT_ENABLE {
                p.int_control = (data & INTCTRL_EI) | (p.int_control & !INTCTRL_EI);
                p.int_enabled = (p.int_control & INTCTRL_EI) != 0;
            }
        }
    }

    fn read_ctrl(&self) -> u8 {
        (self.ports[0].int_control & PORTA_STATUS_MASK)
            | (self.ports[1].int_control >> PORTB_STATUS_SHIFT)
    }

    fn write_data(&mut self, port_id: usize, data: u8) {
        let p = &mut self.ports[port_id];
        match p.mode {
            MODE_OUTPUT | MODE_INPUT | MODE_BITCONTROL => {
                p.output = data;
            }
            MODE_BIDIRECTIONAL => {}
            _ => unreachable!(),
        }
    }

    fn read_data(&self, port_id: usize) -> u8 {
        let p = &self.ports[port_id];
        match p.mode {
            MODE_OUTPUT => p.output,
            MODE_INPUT => p.input,
            MODE_BIDIRECTIONAL => PORT_FLOAT,
            MODE_BITCONTROL => (p.input & p.io_select) | (p.output & !p.io_select),
            _ => unreachable!(),
        }
    }

    fn set_port_output_pins(&self, mut current_pins: u64) -> u64 {
        for (i, p) in self.ports.iter().enumerate() {
            let data = match p.mode {
                MODE_OUTPUT => p.output,
                MODE_INPUT | MODE_BIDIRECTIONAL => PORT_FLOAT,
                MODE_BITCONTROL => p.io_select | (p.output & !p.io_select),
                _ => unreachable!(),
            };

            if i == 0 {
                current_pins = (current_pins & !PA_MASK) | ((data as u64) << PA_SHIFT);
            } else {
                current_pins = (current_pins & !PB_MASK) | ((data as u64) << PB_SHIFT);
            }
        }
        current_pins
    }

    fn handle_iorq(&mut self, mut current_pins: u64) -> u64 {
        let port_id = if (current_pins & BASEL) != 0 { 1 } else { 0 };

        if (current_pins & pins::RD) != 0 {
            let data = if (current_pins & CDSEL) != 0 {
                self.read_ctrl()
            } else {
                self.read_data(port_id)
            };
            current_pins = pins::set_data(current_pins, data);
        } else {
            let data = pins::data(current_pins);
            if (current_pins & CDSEL) != 0 {
                self.write_ctrl(port_id, data);
            } else {
                self.write_data(port_id, data);
            }
        }
        current_pins
    }

    fn handle_int(&mut self, mut current_pins: u64) -> u64 {
        for p in &mut self.ports {
            current_pins = daisy_chain_step(current_pins, &mut p.int_state, p.int_vector);
        }
        current_pins
    }

    fn read_port_inputs(&mut self, current_pins: u64) {
        for (i, p) in self.ports.iter_mut().enumerate() {
            let data = if i == 0 {
                ((current_pins & PA_MASK) >> PA_SHIFT) as u8
            } else {
                ((current_pins & PB_MASK) >> PB_SHIFT) as u8
            };

            if data != p.input || (current_pins & CE) != 0 {
                if p.mode == MODE_INPUT {
                    p.input = data;
                } else if p.mode == MODE_BITCONTROL {
                    p.input = data;
                    let val = (p.input & p.io_select) | (p.output & !p.io_select);
                    let mask = !p.int_mask;
                    let masked_val = val & mask;

                    let ictrl = p.int_control & INT_LOGIC_MASK;
                    let match_found = match ictrl {
                        INT_LOGIC_OR_LOW => masked_val != mask,
                        INT_LOGIC_OR_HIGH => masked_val != 0,
                        INT_LOGIC_AND_LOW => masked_val == 0,
                        INT_LOGIC_AND_HIGH => masked_val == mask,
                        _ => false,
                    };

                    if !p.bctrl_match && match_found && p.int_enabled {
                        p.int_state |= INT_NEEDED;
                    }
                    p.bctrl_match = match_found;
                }
            }
        }
    }

    pub fn tick(&mut self, mut current_pins: u64) -> u64 {
        if (current_pins & (CE | pins::IORQ | pins::M1)) == (CE | pins::IORQ) {
            current_pins = self.handle_iorq(current_pins);
        }
        self.read_port_inputs(current_pins);
        if (current_pins & ASTB) != 0 && (self.pins & ASTB) == 0 {
            let p = &mut self.ports[0];
            if p.mode != MODE_BITCONTROL && p.int_enabled {
                p.int_state |= INT_NEEDED;
            }
        }
        current_pins = self.set_port_output_pins(current_pins);
        current_pins = self.handle_int(current_pins);

        self.pins = current_pins;
        current_pins
    }
}
