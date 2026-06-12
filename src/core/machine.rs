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
const BASIC_START_DELAY_CYCLES: u64 = CLOCK_HZ as u64 / 2;
const KBD_BUF_CHAR: usize = 0x0024;
const KBD_BUF_FLAG: usize = 0x0025;
const KEY_RETURN: u8 = 0x0D;
const SSS_LAUNCH_KEYS: &[u8] = b"BASIC\r";
const SSS_RUN_KEYS: &[u8] = b"RUN\r";
const SCREEN_STABLE_FRAMES: u32 = 8;

#[derive(Clone, Copy)]
enum SssStage {
    Launch(usize),
    AwaitMemPrompt(u64),
    AnswerMem,
    AwaitMemConsumed,
    AwaitInit {
        last: u32,
        stable: u32,
        changed: bool,
    },
    Autorun(usize),
}

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
    pub c80: bool,
    pub rtc: bool,
}

impl Default for Hardware {
    fn default() -> Self {
        Self {
            ram: RamSize::Default,
            chargen: false,
            graphics: GraphicsModule::None,
            c80: false,
            rtc: false,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoadFormat {
    #[default]
    Auto,
    Sss,
}

struct PendingLoad {
    data: Vec<u8>,
    format: LoadFormat,
    autorun: bool,
    at_cycle: u64,
    stage: SssStage,
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

    pub fn schedule_load(&mut self, data: Vec<u8>, format: LoadFormat, autorun: bool) {
        self.pending_load = Some(PendingLoad {
            data,
            format,
            autorun,
            at_cycle: self.total_cycles + BOOT_DELAY_CYCLES,
            stage: SssStage::Launch(0),
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

    fn basic_beg_addr(&self) -> u16 {
        if matches!(self.machine_type, MachineType::KC87) {
            0x0401
        } else {
            0x2C01
        }
    }

    fn feed_key(&mut self, key: u8) {
        self.bus.ram[KBD_BUF_CHAR] = key;
        self.bus.ram[KBD_BUF_FLAG] = key;
    }

    fn screen_checksum(&self) -> u32 {
        self.bus.ram[0xEC00..0xF000]
            .iter()
            .fold(0u32, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u32))
    }

    fn set_sss_stage(&mut self, stage: SssStage) {
        if let Some(p) = self.pending_load.as_mut() {
            p.stage = stage;
        }
    }

    fn advance_sss_load(&mut self, now: u64) {
        let Some(stage) = self.pending_load.as_ref().map(|p| p.stage) else {
            return;
        };
        match stage {
            SssStage::Launch(idx) => {
                if self.bus.ram[KBD_BUF_FLAG] == 0 {
                    self.feed_key(SSS_LAUNCH_KEYS[idx]);
                    let next = idx + 1;
                    self.set_sss_stage(if next < SSS_LAUNCH_KEYS.len() {
                        SssStage::Launch(next)
                    } else {
                        SssStage::AwaitMemPrompt(now)
                    });
                }
            }
            SssStage::AwaitMemPrompt(launched_at) => {
                if now >= launched_at + BASIC_START_DELAY_CYCLES {
                    self.set_sss_stage(SssStage::AnswerMem);
                }
            }
            SssStage::AnswerMem => {
                if self.bus.ram[KBD_BUF_FLAG] == 0 {
                    self.feed_key(KEY_RETURN);
                    self.set_sss_stage(SssStage::AwaitMemConsumed);
                }
            }
            SssStage::AwaitMemConsumed => {
                if self.bus.ram[KBD_BUF_FLAG] == 0 {
                    let last = self.screen_checksum();
                    self.set_sss_stage(SssStage::AwaitInit {
                        last,
                        stable: 0,
                        changed: false,
                    });
                }
            }
            SssStage::AwaitInit {
                last,
                stable,
                changed,
            } => {
                let now_sum = self.screen_checksum();
                if now_sum != last {
                    self.set_sss_stage(SssStage::AwaitInit {
                        last: now_sum,
                        stable: 0,
                        changed: true,
                    });
                } else if changed && stable + 1 >= SCREEN_STABLE_FRAMES {
                    let prep = self
                        .pending_load
                        .as_mut()
                        .map(|p| (std::mem::take(&mut p.data), p.autorun));
                    if let Some((data, autorun)) = prep {
                        let _ = self.load_sss(&data);
                        if autorun {
                            self.set_sss_stage(SssStage::Autorun(0));
                        } else {
                            self.pending_load = None;
                        }
                    }
                } else {
                    self.set_sss_stage(SssStage::AwaitInit {
                        last,
                        stable: stable + 1,
                        changed,
                    });
                }
            }
            SssStage::Autorun(idx) => {
                if self.bus.ram[KBD_BUF_FLAG] == 0 {
                    self.feed_key(SSS_RUN_KEYS[idx]);
                    let next = idx + 1;
                    if next < SSS_RUN_KEYS.len() {
                        self.set_sss_stage(SssStage::Autorun(next));
                    } else {
                        self.pending_load = None;
                    }
                }
            }
        }
    }

    pub fn load_sss(&mut self, payload: &[u8]) -> Result<(), MachineError> {
        let (len_off, data_off) = if payload.len() >= 13
            && payload[0] == payload[1]
            && payload[1] == payload[2]
            && (0xD3..=0xD8).contains(&payload[0])
        {
            (11usize, 13usize)
        } else {
            (0usize, 2usize)
        };
        if payload.len() < data_off {
            return Err(MachineError::FileTooShort);
        }
        let len = u16::from_le_bytes([payload[len_off], payload[len_off + 1]]) as usize;
        let prog = &payload[data_off..];
        let n = len.min(prog.len());
        let beg = self.basic_beg_addr();
        for (i, &byte) in prog[..n].iter().enumerate() {
            self.bus.write_memory(beg.wrapping_add(i as u16), byte);
        }
        let top = beg.wrapping_add(n as u16);
        for off in [42u16, 40, 38] {
            let addr = beg.wrapping_sub(off);
            self.bus.write_memory(addr, (top & 0xFF) as u8);
            self.bus
                .write_memory(addr.wrapping_add(1), (top >> 8) as u8);
        }
        Ok(())
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
        if let Some((true, format)) = self
            .pending_load
            .as_ref()
            .map(|p| (now >= p.at_cycle, p.format))
        {
            match format {
                LoadFormat::Auto => {
                    if let Some(pending) = self.pending_load.take() {
                        let _ = self.load_quick(&pending.data, pending.autorun);
                    }
                }
                LoadFormat::Sss => self.advance_sss_load(now),
            }
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
