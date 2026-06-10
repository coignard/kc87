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
use std::fmt;
use u880::Cpu;

use super::beeper::Beeper;
use super::bus::Bus;
use super::peripherals::UserPeripheral;

pub const MASTER_CLOCK_HZ: u32 = 2_457_600;
pub const CPU_DIVIDER: u32 = 1;
pub const CLOCK_HZ: u32 = MASTER_CLOCK_HZ / CPU_DIVIDER;

pub const FRAME_RATE_HZ: u32 = 50;

pub const DEFAULT_FRAME_CYCLES: u32 = CLOCK_HZ / FRAME_RATE_HZ;
pub const MAX_FRAME_CYCLES: u32 = DEFAULT_FRAME_CYCLES * 2 + 1;
pub const FRAME_TIME_US: u32 = 1_000_000 / FRAME_RATE_HZ;

const BOOT_DELAY_CYCLES: u64 = CLOCK_HZ as u64 * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MachineType {
    Z9001,
    KC87,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum RamSize {
    #[default]
    Default,
    K16,
    K32,
    K48,
    K64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum GraphicsModule {
    #[default]
    None,
    Robotron,
    Krt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Hardware {
    pub ram: RamSize,
    pub chargen: bool,
    pub graphics: GraphicsModule,
}

impl Default for Hardware {
    fn default() -> Self {
        Self {
            ram: RamSize::Default,
            chargen: false,
            graphics: GraphicsModule::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineError {
    InvalidFormat,
    FileTooShort,
    MemoryOverflow,
}

impl fmt::Display for MachineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "Invalid file format"),
            Self::FileTooShort => write!(f, "File is too short"),
            Self::MemoryOverflow => write!(f, "Program exceeds available memory"),
        }
    }
}

impl std::error::Error for MachineError {}

#[derive(Serialize)]
pub struct MachineState<'a> {
    #[serde(rename = "frame")]
    pub cycle: u64,
    pub pc: u16,
    pub bus: &'a Bus,
}

struct PendingLoad {
    data: Vec<u8>,
    autorun: bool,
    at_cycle: u64,
}

pub struct Machine {
    pub cpu: Cpu,
    pub bus: Bus,
    pub beeper: Beeper,
    pub pins: u64,
    pub machine_type: MachineType,
    frame_acc: u32,
    total_cycles: u64,
    pending_load: Option<PendingLoad>,
}

impl Machine {
    pub fn new(
        machine_type: MachineType,
        hardware: Hardware,
        rom_module: Option<Vec<u8>>,
        basic_rom: Vec<u8>,
        os_rom_1: Vec<u8>,
        os_rom_2: Option<Vec<u8>>,
        sample_rate: u32,
    ) -> Self {
        let mut cpu = Cpu::new();
        cpu.reset();
        let pins = cpu.prefetch(0xF000);

        Self {
            cpu,
            bus: Bus::new(
                machine_type,
                hardware,
                rom_module,
                basic_rom,
                os_rom_1,
                os_rom_2,
            ),
            beeper: Beeper::new(CLOCK_HZ, sample_rate, 0.5),
            pins,
            machine_type,
            frame_acc: 0,
            total_cycles: 0,
            pending_load: None,
        }
    }

    pub fn schedule_load(&mut self, data: Vec<u8>, autorun: bool) {
        self.pending_load = Some(PendingLoad {
            data,
            autorun,
            at_cycle: self.total_cycles + BOOT_DELAY_CYCLES,
        });
    }

    pub fn plug_user_peripheral(&mut self, peripheral: UserPeripheral) {
        self.bus.user_slot = peripheral;
    }

    #[inline]
    pub fn drain_midi_out<F: FnMut(&[(u8, u64)])>(&mut self, mut f: F) {
        if let UserPeripheral::Midi(midi) = &mut self.bus.user_slot
            && !midi.out_buffer.is_empty()
        {
            f(&midi.out_buffer);
            midi.out_buffer.clear();
        }
    }

    #[inline]
    pub fn cycle_count(&self) -> u64 {
        self.total_cycles
    }

    pub fn state(&self) -> MachineState<'_> {
        MachineState {
            cycle: self.total_cycles,
            pc: self.cpu.regs.pc,
            bus: &self.bus,
        }
    }

    pub fn start_execution(&mut self, exec_addr: u16) {
        self.cpu.regs.a = 0x00;
        self.cpu.regs.f = u880::Flags::from_bits_retain(0x10);
        self.cpu.regs.set_bc(0x0000);
        self.cpu.regs.set_de(0x0000);
        self.cpu.regs.set_hl(0x0000);
        self.cpu.regs.a2 = 0x00;
        self.cpu.regs.f2 = u880::Flags::empty();
        self.cpu.regs.b2 = 0x00;
        self.cpu.regs.c2 = 0x00;
        self.cpu.regs.d2 = 0x00;
        self.cpu.regs.e2 = 0x00;
        self.cpu.regs.h2 = 0x00;
        self.cpu.regs.l2 = 0x00;
        self.pins = self.cpu.prefetch(exec_addr);
    }

    pub fn validate_kcc(data: &[u8]) -> Result<(u16, u16, Option<u16>), MachineError> {
        if data.len() < 128 {
            return Err(MachineError::FileTooShort);
        }
        if data[..16].iter().any(|&b| b >= 128) {
            return Err(MachineError::InvalidFormat);
        }
        let num_addr = data[16];
        if num_addr > 3 {
            return Err(MachineError::InvalidFormat);
        }
        let load_addr = u16::from_le_bytes([data[17], data[18]]);
        let end_addr = u16::from_le_bytes([data[19], data[20]]);
        if end_addr <= load_addr {
            return Err(MachineError::InvalidFormat);
        }

        let mut exec_addr = None;
        if num_addr > 2 {
            let ea = u16::from_le_bytes([data[21], data[22]]);
            if ea < load_addr || ea > end_addr {
                return Err(MachineError::InvalidFormat);
            }
            exec_addr = Some(ea);
        }

        let required_len = (end_addr - load_addr) as usize + 128;
        if data.len() < required_len {
            return Err(MachineError::FileTooShort);
        }

        Ok((load_addr, end_addr, exec_addr))
    }

    pub fn load_kcc(&mut self, payload: &[u8], autorun: bool) -> Result<(), MachineError> {
        let (load_addr, end_addr, exec_addr) = Self::validate_kcc(payload)?;
        let mut ptr = 128;
        for addr in load_addr..end_addr {
            if ptr < payload.len() {
                self.bus.write_memory(addr, payload[ptr]);
                ptr += 1;
            } else {
                break;
            }
        }
        if autorun {
            if let Some(ea) = exec_addr {
                self.start_execution(ea);
            } else {
                self.start_execution(load_addr);
            }
        }
        Ok(())
    }

    pub fn validate_kctap(data: &[u8]) -> Result<(u16, u16, Option<u16>), MachineError> {
        if data.len() <= 145 {
            return Err(MachineError::FileTooShort);
        }
        let sig = b"\xC3KC-TAPE by AF. ";
        if &data[0..16] != sig {
            return Err(MachineError::InvalidFormat);
        }
        let num_addr = data[17 + 16];
        if num_addr > 3 {
            return Err(MachineError::InvalidFormat);
        }
        let load_addr = u16::from_le_bytes([data[17 + 17], data[17 + 18]]);
        let end_addr = u16::from_le_bytes([data[17 + 19], data[17 + 20]]);
        if end_addr <= load_addr {
            return Err(MachineError::InvalidFormat);
        }

        let mut exec_addr = None;
        if num_addr > 2 {
            let ea = u16::from_le_bytes([data[17 + 21], data[17 + 22]]);
            if ea < load_addr || ea > end_addr {
                return Err(MachineError::InvalidFormat);
            }
            exec_addr = Some(ea);
        }

        let required_len = (end_addr - load_addr) as usize + 145;
        if data.len() < required_len {
            return Err(MachineError::FileTooShort);
        }

        Ok((load_addr, end_addr, exec_addr))
    }

    pub fn load_kctap(&mut self, payload: &[u8], autorun: bool) -> Result<(), MachineError> {
        let (load_addr, end_addr, exec_addr) = Self::validate_kctap(payload)?;
        let mut addr = load_addr;
        let mut ptr = 145;

        while addr < end_addr {
            if ptr >= payload.len() {
                break;
            }
            ptr += 1;
            for _ in 0..128 {
                if ptr >= payload.len() {
                    break;
                }
                if addr < end_addr {
                    self.bus.write_memory(addr, payload[ptr]);
                    addr += 1;
                }
                ptr += 1;
            }
        }

        if autorun {
            if let Some(ea) = exec_addr {
                self.start_execution(ea);
            } else {
                self.start_execution(load_addr);
            }
        }
        Ok(())
    }

    pub fn load_quick(&mut self, payload: &[u8], autorun: bool) -> Result<(), MachineError> {
        if Self::validate_kctap(payload).is_ok() {
            self.load_kctap(payload, autorun)
        } else if Self::validate_kcc(payload).is_ok() {
            self.load_kcc(payload, autorun)
        } else {
            Err(MachineError::InvalidFormat)
        }
    }

    #[inline(always)]
    pub fn key_down(&mut self, key: i32) {
        self.bus.keyboard.key_down(key);
    }

    #[inline(always)]
    pub fn key_up(&mut self, key: i32) {
        self.bus.keyboard.key_up(key);
    }

    pub fn tick<S>(&mut self, mut push_sample: S) -> bool
    where
        S: FnMut(f32),
    {
        let mut vblank_occurred = false;
        let mut frame_cycles = 0;

        let now = self.total_cycles;
        if self
            .pending_load
            .as_ref()
            .is_some_and(|p| now >= p.at_cycle)
            && let Some(pending) = self.pending_load.take()
        {
            let _ = self.load_quick(&pending.data, pending.autorun);
        }

        while !vblank_occurred && frame_cycles < MAX_FRAME_CYCLES {
            self.pins = self.cpu.tick(self.pins);
            self.bus.current_cycle = self.total_cycles;
            let (new_pins, beeper_toggled) = self.bus.tick(self.pins);
            self.pins = new_pins;

            if beeper_toggled {
                self.beeper.toggle();
            }

            if self.beeper.tick(self.bus.audio_enabled()) {
                push_sample(self.beeper.sample);
            }

            frame_cycles += 1;
            self.total_cycles += 1;

            self.frame_acc += 1;
            if self.frame_acc >= DEFAULT_FRAME_CYCLES {
                self.frame_acc -= DEFAULT_FRAME_CYCLES;
                vblank_occurred = true;
            }
        }

        if vblank_occurred {
            self.bus.keyboard.update(FRAME_TIME_US);
        }

        vblank_occurred
    }
}
