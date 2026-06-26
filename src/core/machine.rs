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
use super::bus::{Bus, memory_map};
use super::peripherals::UserPeripheral;
use super::peripherals::keyboard::Key;

pub const MASTER_CLOCK_HZ: u32 = 2_457_600;
pub const CPU_DIVIDER: u32 = 1;
pub const CLOCK_HZ: u32 = MASTER_CLOCK_HZ / CPU_DIVIDER;

pub const FRAME_RATE_HZ: u32 = 50;

pub const DEFAULT_FRAME_CYCLES: u32 = CLOCK_HZ / FRAME_RATE_HZ;
pub const MAX_FRAME_CYCLES: u32 = DEFAULT_FRAME_CYCLES * 2 + 1;
pub const FRAME_TIME_US: u32 = 1_000_000 / FRAME_RATE_HZ;

const BOOT_DELAY_CYCLES: u64 = CLOCK_HZ as u64 * 2;
const KBD_BUF_CHAR: usize = 0x0024;
const KBD_BUF_FLAG: usize = 0x0025;
const KEY_RETURN: u8 = 0x0D;
const SSS_LAUNCH_KEYS: &[u8] = b"BASIC\r";
const SSS_RUN_KEYS: &[u8] = b"RUN\r";
const KBD_WAIT_LOOP_BEG: u16 = 0xF924;
const KBD_WAIT_LOOP_END: u16 = 0xF92A;
const SCREEN_STABLE_FRAMES: u32 = 8;
const SSS_HEADER_LEN: usize = 11;
const SSS_PROG_LEN_FIELD: usize = 2;
const SSS_HEADER_ID_MIN: u8 = 0xD3;
const SSS_HEADER_ID_MAX: u8 = 0xD8;
const BASIC_END_PTR_OFFSETS: [u16; 3] = [42, 40, 38];

const RESET_VECTOR: u16 = 0xF000;
const ROM_BASIC_BEG: u16 = 0x0401;
const RAM_BASIC_BEG: u16 = 0x2C01;
const START_FLAGS: u8 = 0x10;
const SCREEN_HASH_MULT: u32 = 31;

const KCC_HEADER_LEN: usize = 128;
const KCC_NAME_LEN: usize = 16;
const KCC_NAME_ASCII_LIMIT: u8 = 0x80;
const KCC_MAX_ADDR_COUNT: u8 = 3;
const KCC_NUM_ADDR_OFF: usize = 16;
const KCC_LOAD_ADDR_OFF: usize = 17;
const KCC_END_ADDR_OFF: usize = 19;
const KCC_EXEC_ADDR_OFF: usize = 21;

const KCTAP_SIG: &[u8] = b"\xC3KC-TAPE by AF. ";
const KCTAP_SIG_LEN: usize = 16;
const KCTAP_BLOCK_PREFIX: usize = 17;
const KCTAP_BLOCK_LEN: usize = 128;
const KCTAP_MIN_LEN: usize = KCTAP_BLOCK_PREFIX + KCC_HEADER_LEN;

#[derive(Clone, Copy)]
enum SssStage {
    AwaitBoot,
    Launch(usize),
    AwaitMemPrompt {
        last: u32,
        stable: u32,
        changed: bool,
    },
    AnswerMem(usize),
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

#[derive(Clone)]
pub struct ModulePreload {
    pub data: Vec<u8>,
    pub addr: Option<u16>,
}

impl ModulePreload {
    pub fn headered(data: Vec<u8>) -> Self {
        Self { data, addr: None }
    }

    pub fn raw(data: Vec<u8>, addr: u16) -> Self {
        Self {
            data,
            addr: Some(addr),
        }
    }
}

struct PendingLoad {
    data: Vec<u8>,
    format: LoadFormat,
    autorun: bool,
    at_cycle: u64,
    stage: SssStage,
    modules: Vec<ModulePreload>,
    mem_answer: Vec<u8>,
    enter_basic_only: bool,
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
        let pins = cpu.prefetch(RESET_VECTOR);

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

    fn mem_answer_for(modules: &[ModulePreload]) -> Vec<u8> {
        let lowest = modules.iter().filter_map(Self::module_load_addr).min();
        match lowest {
            Some(addr) => format!("{}\r", addr.wrapping_sub(1)).into_bytes(),
            None => vec![KEY_RETURN],
        }
    }

    fn module_load_addr(module: &ModulePreload) -> Option<u16> {
        module.addr.or_else(|| {
            Self::validate_kctap(&module.data)
                .or_else(|_| Self::validate_kcc(&module.data))
                .ok()
                .map(|(load_addr, _, _)| load_addr)
        })
    }

    fn preload_module(&mut self, module: &ModulePreload) {
        match module.addr {
            Some(addr) => {
                for (i, &byte) in module.data.iter().enumerate() {
                    self.bus.write_memory(addr.wrapping_add(i as u16), byte);
                }
            }
            None => {
                let _ = self.load_quick(&module.data, false);
            }
        }
    }

    pub fn schedule_load(
        &mut self,
        data: Vec<u8>,
        format: LoadFormat,
        autorun: bool,
        modules: Vec<ModulePreload>,
    ) {
        let mem_answer = Self::mem_answer_for(&modules);
        let (at_cycle, stage) = match format {
            LoadFormat::Sss => (self.total_cycles, SssStage::AwaitBoot),
            LoadFormat::Auto => (self.total_cycles + BOOT_DELAY_CYCLES, SssStage::Launch(0)),
        };
        self.pending_load = Some(PendingLoad {
            data,
            format,
            autorun,
            at_cycle,
            stage,
            modules,
            mem_answer,
            enter_basic_only: false,
        });
    }

    pub fn schedule_basic_autostart(&mut self, modules: Vec<ModulePreload>) {
        let mem_answer = Self::mem_answer_for(&modules);
        self.pending_load = Some(PendingLoad {
            data: Vec::new(),
            format: LoadFormat::Sss,
            autorun: false,
            at_cycle: self.total_cycles,
            stage: SssStage::AwaitBoot,
            modules,
            mem_answer,
            enter_basic_only: true,
        });
    }

    pub fn schedule_modules_preload(&mut self, modules: Vec<ModulePreload>) {
        self.pending_load = Some(PendingLoad {
            data: Vec::new(),
            format: LoadFormat::Auto,
            autorun: false,
            at_cycle: self.total_cycles + BOOT_DELAY_CYCLES,
            stage: SssStage::Launch(0),
            modules,
            mem_answer: vec![KEY_RETURN],
            enter_basic_only: false,
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
        self.cpu.regs.f = u880::Flags::from_bits_retain(START_FLAGS);
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
        if data.len() < KCC_HEADER_LEN {
            return Err(MachineError::FileTooShort);
        }
        if data[..KCC_NAME_LEN]
            .iter()
            .any(|&b| b >= KCC_NAME_ASCII_LIMIT)
        {
            return Err(MachineError::InvalidFormat);
        }
        let num_addr = data[KCC_NUM_ADDR_OFF];
        if num_addr > KCC_MAX_ADDR_COUNT {
            return Err(MachineError::InvalidFormat);
        }
        let load_addr = u16::from_le_bytes([data[KCC_LOAD_ADDR_OFF], data[KCC_LOAD_ADDR_OFF + 1]]);
        let end_addr = u16::from_le_bytes([data[KCC_END_ADDR_OFF], data[KCC_END_ADDR_OFF + 1]]);
        if end_addr <= load_addr {
            return Err(MachineError::InvalidFormat);
        }

        let ea = u16::from_le_bytes([data[KCC_EXEC_ADDR_OFF], data[KCC_EXEC_ADDR_OFF + 1]]);
        let exec_addr = if ea != 0 && ea != 0xFFFF {
            Some(ea)
        } else {
            None
        };

        let required_len = (end_addr - load_addr) as usize + KCC_HEADER_LEN;
        if data.len() < required_len {
            return Err(MachineError::FileTooShort);
        }

        Ok((load_addr, end_addr, exec_addr))
    }

    pub fn load_kcc(&mut self, payload: &[u8], autorun: bool) -> Result<(), MachineError> {
        let (load_addr, end_addr, exec_addr) = Self::validate_kcc(payload)?;
        let mut ptr = KCC_HEADER_LEN;
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
        if data.len() <= KCTAP_MIN_LEN {
            return Err(MachineError::FileTooShort);
        }
        if &data[0..KCTAP_SIG_LEN] != KCTAP_SIG {
            return Err(MachineError::InvalidFormat);
        }
        let num_addr = data[KCTAP_BLOCK_PREFIX + KCC_NUM_ADDR_OFF];
        if num_addr > KCC_MAX_ADDR_COUNT {
            return Err(MachineError::InvalidFormat);
        }
        let load_addr = u16::from_le_bytes([
            data[KCTAP_BLOCK_PREFIX + KCC_LOAD_ADDR_OFF],
            data[KCTAP_BLOCK_PREFIX + KCC_LOAD_ADDR_OFF + 1],
        ]);
        let end_addr = u16::from_le_bytes([
            data[KCTAP_BLOCK_PREFIX + KCC_END_ADDR_OFF],
            data[KCTAP_BLOCK_PREFIX + KCC_END_ADDR_OFF + 1],
        ]);
        if end_addr <= load_addr {
            return Err(MachineError::InvalidFormat);
        }

        let ea = u16::from_le_bytes([
            data[KCTAP_BLOCK_PREFIX + KCC_EXEC_ADDR_OFF],
            data[KCTAP_BLOCK_PREFIX + KCC_EXEC_ADDR_OFF + 1],
        ]);
        let exec_addr = if ea != 0 && ea != 0xFFFF {
            Some(ea)
        } else {
            None
        };

        let required_len = (end_addr - load_addr) as usize + KCTAP_MIN_LEN;
        if data.len() < required_len {
            return Err(MachineError::FileTooShort);
        }

        Ok((load_addr, end_addr, exec_addr))
    }

    pub fn load_kctap(&mut self, payload: &[u8], autorun: bool) -> Result<(), MachineError> {
        let (load_addr, end_addr, exec_addr) = Self::validate_kctap(payload)?;
        let mut addr = load_addr;
        let mut ptr = KCTAP_MIN_LEN;

        while addr < end_addr {
            if ptr >= payload.len() {
                break;
            }
            ptr += 1;
            for _ in 0..KCTAP_BLOCK_LEN {
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
            ROM_BASIC_BEG
        } else {
            RAM_BASIC_BEG
        }
    }

    fn feed_key(&mut self, key: u8) {
        self.bus.ram[KBD_BUF_CHAR] = key;
        self.bus.ram[KBD_BUF_FLAG] = key;
    }

    fn screen_checksum(&self) -> u32 {
        self.bus.ram[memory_map::VIDEO_RAM_START as usize..memory_map::VIDEO_RAM_END as usize]
            .iter()
            .fold(0u32, |acc, &b| {
                acc.wrapping_mul(SCREEN_HASH_MULT).wrapping_add(b as u32)
            })
    }

    fn poll_screen_settled(&self, last: u32, stable: u32, changed: bool) -> Option<(u32, u32, bool)> {
        let now_sum = self.screen_checksum();
        if now_sum != last {
            Some((now_sum, 0, true))
        } else if changed && stable + 1 >= SCREEN_STABLE_FRAMES {
            None
        } else {
            Some((last, stable + 1, changed))
        }
    }

    fn set_sss_stage(&mut self, stage: SssStage) {
        if let Some(p) = self.pending_load.as_mut() {
            p.stage = stage;
        }
    }

    fn advance_sss_load(&mut self) {
        let Some(stage) = self.pending_load.as_ref().map(|p| p.stage) else {
            return;
        };
        match stage {
            SssStage::AwaitBoot => {
                let pc = self.cpu.regs.pc;
                if (KBD_WAIT_LOOP_BEG..KBD_WAIT_LOOP_END).contains(&pc)
                    && self.bus.ram[KBD_BUF_FLAG] == 0
                {
                    self.feed_key(SSS_LAUNCH_KEYS[0]);
                    self.set_sss_stage(SssStage::Launch(1));
                }
            }
            SssStage::Launch(idx) => {
                if self.bus.ram[KBD_BUF_FLAG] == 0 {
                    self.feed_key(SSS_LAUNCH_KEYS[idx]);
                    let next = idx + 1;
                    let stage = if next < SSS_LAUNCH_KEYS.len() {
                        SssStage::Launch(next)
                    } else {
                        SssStage::AwaitMemPrompt {
                            last: self.screen_checksum(),
                            stable: 0,
                            changed: false,
                        }
                    };
                    self.set_sss_stage(stage);
                }
            }
            SssStage::AwaitMemPrompt {
                last,
                stable,
                changed,
            } => match self.poll_screen_settled(last, stable, changed) {
                Some((last, stable, changed)) => {
                    self.set_sss_stage(SssStage::AwaitMemPrompt {
                        last,
                        stable,
                        changed,
                    });
                }
                None => self.set_sss_stage(SssStage::AnswerMem(0)),
            },
            SssStage::AnswerMem(idx) => {
                if self.bus.ram[KBD_BUF_FLAG] == 0 {
                    let answer = self
                        .pending_load
                        .as_ref()
                        .map(|p| (p.mem_answer.get(idx).copied(), p.mem_answer.len()));
                    if let Some((Some(key), len)) = answer {
                        self.feed_key(key);
                        let next = idx + 1;
                        self.set_sss_stage(if next < len {
                            SssStage::AnswerMem(next)
                        } else {
                            SssStage::AwaitMemConsumed
                        });
                    } else {
                        self.set_sss_stage(SssStage::AwaitMemConsumed);
                    }
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
            } => match self.poll_screen_settled(last, stable, changed) {
                Some((last, stable, changed)) => {
                    self.set_sss_stage(SssStage::AwaitInit {
                        last,
                        stable,
                        changed,
                    });
                }
                None => {
                    let prep = self.pending_load.as_mut().map(|p| {
                        (
                            std::mem::take(&mut p.data),
                            p.autorun,
                            std::mem::take(&mut p.modules),
                            p.enter_basic_only,
                        )
                    });
                    if let Some((data, autorun, modules, basic_only)) = prep {
                        for module in &modules {
                            self.preload_module(module);
                        }
                        if basic_only {
                            self.pending_load = None;
                        } else {
                            let _ = self.load_sss(&data);
                            if autorun {
                                self.set_sss_stage(SssStage::Autorun(0));
                            } else {
                                self.pending_load = None;
                            }
                        }
                    }
                }
            },
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
        let (len_off, data_off) = if payload.len() >= SSS_HEADER_LEN + SSS_PROG_LEN_FIELD
            && payload[0] == payload[1]
            && payload[1] == payload[2]
            && (SSS_HEADER_ID_MIN..=SSS_HEADER_ID_MAX).contains(&payload[0])
        {
            (SSS_HEADER_LEN, SSS_HEADER_LEN + SSS_PROG_LEN_FIELD)
        } else {
            (0, SSS_PROG_LEN_FIELD)
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
        for off in BASIC_END_PTR_OFFSETS {
            let addr = beg.wrapping_sub(off);
            self.bus.write_memory(addr, (top & 0xFF) as u8);
            self.bus
                .write_memory(addr.wrapping_add(1), (top >> 8) as u8);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn key_down(&mut self, key: Key) {
        self.bus.keyboard.key_down(key);
    }

    #[inline(always)]
    pub fn key_up(&mut self, key: Key) {
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
                    let autorun = self
                        .pending_load
                        .as_ref()
                        .map(|p| p.autorun)
                        .unwrap_or(false);
                    let at_safe_point =
                        (KBD_WAIT_LOOP_BEG..KBD_WAIT_LOOP_END).contains(&self.cpu.regs.pc);
                    if !autorun || at_safe_point {
                        if let Some(pending) = self.pending_load.take() {
                            for module in &pending.modules {
                                self.preload_module(module);
                            }
                            if !pending.data.is_empty() {
                                let _ = self.load_quick(&pending.data, pending.autorun);
                            }
                        }
                    }
                }
                LoadFormat::Sss => self.advance_sss_load(),
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
