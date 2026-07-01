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
use sha2::{Digest, Sha256};
use u880::pins;

use crossbeam_channel::Receiver;

use crate::core::chips::rtc7242x::Rtc7242x;
use crate::core::chips::u855::{self, U855};
use crate::core::chips::u857::{self, U857};
use crate::core::chips::u8272::U8272;
use crate::core::machine::{
    FRAME_RATE_HZ, GraphicsModule, Hardware, MASTER_CLOCK_HZ, MachineType, RamSize,
};
use crate::core::peripherals::UserPeripheral;
use crate::core::peripherals::disk::DiskBackend;
use crate::core::peripherals::keyboard::Keyboard;
use crate::core::peripherals::tape::TapeTract;

const CURSOR_BLINK_INTERVAL_MS: u32 = 200;
const BLINK_TOGGLE_CYCLES: u32 = MASTER_CLOCK_HZ * CURSOR_BLINK_INTERVAL_MS / 1000;
const BLINK_FLIP_FLOP_BIT: u8 = 0x80;

const VIDEO_TOTAL_LINES: u32 = 312;
const VIDEO_VISIBLE_LINES: u32 = 192;
const TSTATES_PER_LINE: u32 = MASTER_CLOCK_HZ / FRAME_RATE_HZ / VIDEO_TOTAL_LINES;
const TSTATES_VISIBLE: u32 = TSTATES_PER_LINE.div_ceil(2);

pub mod memory_map {
    pub const RAM_START: u16 = 0x0000;
    pub const RAM_EXT_SEG1_START: u16 = 0x4000;
    pub const RAM_EXT_SEG1_END: u16 = 0x8000;
    pub const UPPER_RAM_START: u16 = 0xC000;
    pub const COLOR_RAM_START: u16 = 0xE800;
    pub const COLOR_RAM_END: u16 = 0xEC00;
    pub const VIDEO_RAM_START: u16 = 0xEC00;
    pub const VIDEO_RAM_END: u16 = 0xF000;
    pub const VIDEO_RAM_SIZE: usize = 0x0400;
    pub const CHARGEN_WINDOW_ON: u16 = 0xEBFC;
    pub const CHARGEN_FONT_ON: u16 = 0xEBFE;
    pub const CHARGEN_OFF: u16 = 0xEBFF;
}

mod ports {
    pub const RAM64K_SEG1_OFF: u8 = 0x04;
    pub const RAM64K_SEG1_ON: u8 = 0x05;
    pub const RAM64K_C000_OFF: u8 = 0x06;
    pub const RAM64K_C000_ON: u8 = 0x07;
    pub const RTC_BASE: u8 = 0x60;
    pub const RTC_BASE_MASK: u8 = 0xF0;
    pub const RTC_REG_MASK: u8 = 0x0F;
    pub const C80_SWAP_OFF: u8 = 0xA0;
    pub const C80_SWAP_ON: u8 = 0xA1;
    pub const C80_WIDE_OFF: u8 = 0xA8;
    pub const C80_WIDE_ON: u8 = 0xA9;
    pub const C80_WIDE_OFF_ALT: u8 = 0xBC;
    pub const C80_WIDE_ON_ALT: u8 = 0xBD;
    pub const C80_SWAP_OFF_ALT: u8 = 0xBE;
    pub const C80_SWAP_ON_ALT: u8 = 0xBF;
    pub const GRAPH_CONTROL: u8 = 0xB8;
    pub const GRAPH_ADDR_LOW: u8 = 0xB9;
    pub const GRAPH_PIXEL: u8 = 0xBA;
    pub const FDC_BASE: u8 = 0x98;
    pub const FDC_PORT_MASK: u8 = 0xF8;
    pub const FDC_DATA_SELECT: u8 = 0x01;
    pub const FDC_CONTROL_BASE: u8 = 0xA0;
    pub const FDC_CONTROL_TC: u8 = 0x10;
    pub const FDC_CONTROL_RESET: u8 = 0x20;
    pub const EA_BASE_C8: u8 = 0xC8;
    pub const EA_PORT_MASK: u8 = 0xFC;
}

const RAM_LAYER: usize = 0;
const ROM_LAYER: usize = 1;
const RAM_BASE: usize = 0;

mod firmware {
    pub const BASIC_SRC: usize = 0x0000;
    pub const BASIC_DEST: u16 = 0xC000;
    pub const Z9001_BASIC_SIZE: u32 = 0x2800;
    pub const Z9001_OS_BANK_SIZE: u32 = 0x0800;
    pub const Z9001_OS1_DEST: u16 = 0xF000;
    pub const Z9001_OS1_SRC: usize = 0x3000;
    pub const Z9001_OS2_DEST: u16 = 0xF800;
    pub const Z9001_OS2_SRC: usize = 0x3800;
    pub const KC87_ROM_BANK_SIZE: u32 = 0x2000;
    pub const KC87_OS_DEST: u16 = 0xE000;
    pub const KC87_OS_SRC: usize = 0x2000;
}

const PORT_ADDR_MASK: u16 = 0x00FF;
const PAGE_SIZE: usize = 1024;
const PAGE_SHIFT: u16 = 10;
const PAGE_OFFSET_MASK: u16 = 0x03FF;
const NUM_PAGES: usize = 64;
const MEM_LAYERS: usize = 4;
const ADDR_WRAP_MASK: usize = 0xFFFF;
const OPEN_BUS_VALUE: u8 = 0xFF;

const COLOR_INDEX_MASK: u8 = 0x07;
const COLOR_NIBBLE_SHIFT: u8 = 4;
const SYS_PORTA_AUDIO_BIT: u8 = 0x80;
const SYS_PORTA_MODE20_BIT: u8 = 0x04;
const SYS_PORTA_BORDER_SHIFT: u8 = 3;
const GRAPH_MODE_BIT: u8 = 0x08;
const GRAPH_BORDER_WRITE_BIT: u8 = 0x80;
const GRAPH_BORDER_READ_BIT: u8 = 0x40;

const PIO_PORT_A_SHIFT: u64 = 48;
const PIO_PORT_B_SHIFT: u64 = 56;

const KBD_STICKY_FRAMES: u32 = 3;

const RAM_TOP_16K: u32 = 0x4000;
const RAM_TOP_32K: u32 = 0x8000;
const RAM_TOP_48K: u32 = 0xC000;

const FDC_MHZ: u32 = 4;
const MILLIS_PER_SECOND: u32 = 1000;

fn serialize_ram_hash<S: serde::Serializer>(
    ram: &[u8; 65536],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let hash = Sha256::digest(ram);
    serializer.serialize_str(&hex::encode(hash))
}

fn serialize_chargen_hash<S: serde::Serializer>(
    font: &[u8; 0x400],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let hash = Sha256::digest(font);
    serializer.serialize_str(&hex::encode(hash))
}

fn serialize_ram_ext_hash<S: serde::Serializer>(
    ram_ext: &[u8; 0x4000],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let hash = Sha256::digest(ram_ext);
    serializer.serialize_str(&hex::encode(hash))
}

fn serialize_ram_pixel_hash<S: serde::Serializer>(
    ram_pixel: &[u8; 0x2000],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let hash = Sha256::digest(ram_pixel);
    serializer.serialize_str(&hex::encode(hash))
}

fn serialize_ram_video2_hash<S: serde::Serializer>(
    ram_video2: &[u8; 0x400],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let hash = Sha256::digest(ram_video2);
    serializer.serialize_str(&hex::encode(hash))
}

fn serialize_ram_color2_hash<S: serde::Serializer>(
    ram_color2: &[u8; 0x400],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let hash = Sha256::digest(ram_color2);
    serializer.serialize_str(&hex::encode(hash))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageRef {
    Ram(usize),
    Rom(usize),
    Unmapped,
}

#[derive(Clone, Copy)]
pub struct MemPage {
    pub read: PageRef,
    pub write: PageRef,
}

impl Default for MemPage {
    fn default() -> Self {
        Self {
            read: PageRef::Unmapped,
            write: PageRef::Unmapped,
        }
    }
}

pub struct Mem {
    pub page_table: [MemPage; NUM_PAGES],
    pub layers: [[MemPage; NUM_PAGES]; MEM_LAYERS],
}

impl Default for Mem {
    fn default() -> Self {
        Self::new()
    }
}

impl Mem {
    pub fn new() -> Self {
        Self {
            page_table: [MemPage::default(); NUM_PAGES],
            layers: [[MemPage::default(); NUM_PAGES]; MEM_LAYERS],
        }
    }

    pub fn update_page_table(&mut self, page_index: usize) {
        for layer in 0..MEM_LAYERS {
            let p = self.layers[layer][page_index];
            if p.read != PageRef::Unmapped || p.write != PageRef::Unmapped {
                self.page_table[page_index] = p;
                return;
            }
        }
        self.page_table[page_index] = MemPage::default();
    }

    pub fn map(
        &mut self,
        layer: usize,
        addr: u16,
        size: u32,
        read_base: PageRef,
        write_base: PageRef,
    ) {
        let num_pages = (size >> PAGE_SHIFT) as usize;
        for i in 0..num_pages {
            let offset = i * PAGE_SIZE;
            let page_index = (((addr as usize) + offset) & ADDR_WRAP_MASK) >> PAGE_SHIFT;

            let r = match read_base {
                PageRef::Ram(b) => PageRef::Ram(b + offset),
                PageRef::Rom(b) => PageRef::Rom(b + offset),
                PageRef::Unmapped => PageRef::Unmapped,
            };
            let w = match write_base {
                PageRef::Ram(b) => PageRef::Ram(b + offset),
                PageRef::Rom(b) => PageRef::Rom(b + offset),
                PageRef::Unmapped => PageRef::Unmapped,
            };

            self.layers[layer][page_index] = MemPage { read: r, write: w };
            self.update_page_table(page_index);
        }
    }

    pub fn map_ram(&mut self, layer: usize, addr: u16, size: u32, base: usize) {
        self.map(layer, addr, size, PageRef::Ram(base), PageRef::Ram(base));
    }

    pub fn map_rom(&mut self, layer: usize, addr: u16, size: u32, base: usize) {
        self.map(layer, addr, size, PageRef::Rom(base), PageRef::Unmapped);
    }
}

#[derive(Serialize)]
pub struct Bus {
    #[serde(serialize_with = "serialize_ram_hash", rename = "ram_hash")]
    pub ram: Box<[u8; 65536]>,
    #[serde(skip)]
    pub rom: Box<[u8; 16384]>,
    #[serde(skip)]
    pub mem: Mem,

    pub pio1: U855,
    pub pio2: U855,
    pub ctc: U857,

    #[serde(skip)]
    pub rtc: Option<Rtc7242x>,

    #[serde(skip)]
    pub fdc: Option<U8272>,
    #[serde(skip)]
    fdc_terminal_count: bool,
    #[serde(skip)]
    fdc_reset_line: bool,

    pub keyboard: Keyboard,

    pub user_slot: UserPeripheral,

    pub blink_flip_flop: u8,
    pub blink_counter: u32,
    pub ctc_zcto2: u64,

    pub sys_porta: u8,

    #[serde(skip)]
    pub chargen: bool,
    #[serde(serialize_with = "serialize_chargen_hash", rename = "chargen_ram_hash")]
    pub chargen_ram: Box<[u8; 0x400]>,
    pub chargen_window: bool,
    pub chargen_active: bool,

    #[serde(skip)]
    pub ram64k_module: bool,
    #[serde(serialize_with = "serialize_ram_ext_hash", rename = "ram_ext_hash")]
    pub ram_ext: Box<[u8; 0x4000]>,
    pub ram_4000_seg1: bool,
    pub ram_c000: bool,
    #[serde(skip)]
    c000_ram_end: u16,

    #[serde(skip)]
    pub rom_module: Option<Box<[u8]>>,
    #[serde(skip)]
    rom_module_end: u16,

    #[serde(skip)]
    has_overlays: bool,

    #[serde(skip)]
    pub graph_type: GraphicsModule,
    #[serde(serialize_with = "serialize_ram_pixel_hash", rename = "ram_pixel_hash")]
    pub ram_pixel: Box<[u8; 0x2000]>,
    pub graph_mode: bool,
    pub graph_fg: u8,
    pub graph_bg: u8,
    pub graph_border: bool,
    pub graph_bank: u8,
    pub graph_addr_l: u8,

    #[serde(skip)]
    pub c80_enabled: bool,
    #[serde(
        serialize_with = "serialize_ram_video2_hash",
        rename = "ram_video2_hash"
    )]
    pub ram_video2: Box<[u8; 0x400]>,
    #[serde(
        serialize_with = "serialize_ram_color2_hash",
        rename = "ram_color2_hash"
    )]
    pub ram_color2: Box<[u8; 0x400]>,
    pub c80_active: bool,
    pub c80_memswap: bool,

    #[serde(skip)]
    pub current_cycle: u64,
    #[serde(skip)]
    prev_user_write: bool,
    #[serde(skip)]
    pub pio_ea: U855,
    #[serde(skip)]
    pub ea_present: bool,
    #[serde(skip)]
    pub ea_slot: UserPeripheral,
    #[serde(skip)]
    prev_ea_write: bool,
    #[serde(skip)]
    line_tstates: u32,
    #[serde(skip)]
    line_num: u32,
    #[serde(skip)]
    wait_pending: u32,
    #[serde(skip)]
    tape_rx: Option<Receiver<f32>>,
    #[serde(skip)]
    tape_sample_rate: u32,
    #[serde(skip)]
    tape_tract: TapeTract,
    #[serde(skip)]
    tape_acc: u32,
}

impl Bus {
    pub fn new(
        machine_type: MachineType,
        hardware: Hardware,
        rom_module: Option<Vec<u8>>,
        basic_rom: Vec<u8>,
        os_rom_1: Vec<u8>,
        os_rom_2: Option<Vec<u8>>,
    ) -> Self {
        let mut full_ram = vec![0u8; 65536];
        let mut r = 0x52656EE9u32;
        let mut i = 0;

        while i < 65536 {
            r ^= r << 13;
            r ^= r >> 17;
            r ^= r << 5;
            full_ram[i] = r as u8;
            full_ram[i + 1] = (r >> 8) as u8;
            full_ram[i + 2] = (r >> 16) as u8;
            full_ram[i + 3] = (r >> 24) as u8;
            i += 4;
        }

        let ram: Box<[u8; 65536]> = full_ram.into_boxed_slice().try_into().unwrap();
        let mut rom: Box<[u8; 16384]> = vec![0u8; 16384].into_boxed_slice().try_into().unwrap();
        let mut mem = Mem::new();

        let load_rom = |rom: &mut [u8; 16384], at: usize, src: &[u8]| {
            let n = src.len().min(rom.len().saturating_sub(at));
            rom[at..at + n].copy_from_slice(&src[..n]);
        };

        let mut keyboard = Keyboard::new(KBD_STICKY_FRAMES);
        keyboard.register_default_layout();

        let ram_top: u32 = match hardware.ram {
            RamSize::K16 => RAM_TOP_16K,
            RamSize::K32 => RAM_TOP_32K,
            RamSize::K48 | RamSize::K64 => RAM_TOP_48K,
            RamSize::Default => match machine_type {
                MachineType::Z9001 => RAM_TOP_32K,
                MachineType::KC87 => RAM_TOP_48K,
            },
        };

        let ram64k_module = hardware.ram == RamSize::K64;
        let c000_ram_end: u16 = memory_map::COLOR_RAM_START;

        let (rom_module, rom_module_end) = match rom_module {
            Some(bytes) if !bytes.is_empty() => {
                let end = (memory_map::UPPER_RAM_START as u32 + bytes.len() as u32)
                    .min(c000_ram_end as u32) as u16;
                (Some(bytes.into_boxed_slice()), end)
            }
            _ => (None, memory_map::UPPER_RAM_START),
        };

        let has_overlays = hardware.chargen
            || ram64k_module
            || rom_module.is_some()
            || hardware.c80
            || hardware.graphics != GraphicsModule::None;

        if machine_type == MachineType::Z9001 {
            if !basic_rom.is_empty() {
                load_rom(&mut rom, firmware::BASIC_SRC, &basic_rom);
                mem.map_rom(
                    ROM_LAYER,
                    firmware::BASIC_DEST,
                    firmware::Z9001_BASIC_SIZE,
                    firmware::BASIC_SRC,
                );
            }
            load_rom(&mut rom, firmware::Z9001_OS1_SRC, &os_rom_1);
            mem.map_rom(
                ROM_LAYER,
                firmware::Z9001_OS1_DEST,
                firmware::Z9001_OS_BANK_SIZE,
                firmware::Z9001_OS1_SRC,
            );
            if let Some(os_2) = os_rom_2 {
                load_rom(&mut rom, firmware::Z9001_OS2_SRC, &os_2);
                mem.map_rom(
                    ROM_LAYER,
                    firmware::Z9001_OS2_DEST,
                    firmware::Z9001_OS_BANK_SIZE,
                    firmware::Z9001_OS2_SRC,
                );
            }
            mem.map_ram(RAM_LAYER, memory_map::RAM_START, ram_top, RAM_BASE);
        } else {
            load_rom(&mut rom, firmware::BASIC_SRC, &basic_rom);
            load_rom(&mut rom, firmware::KC87_OS_SRC, &os_rom_1);
            mem.map_ram(RAM_LAYER, memory_map::RAM_START, ram_top, RAM_BASE);
            mem.map_ram(
                RAM_LAYER,
                memory_map::COLOR_RAM_START,
                memory_map::VIDEO_RAM_SIZE as u32,
                memory_map::COLOR_RAM_START as usize,
            );
            mem.map_rom(
                ROM_LAYER,
                firmware::BASIC_DEST,
                firmware::KC87_ROM_BANK_SIZE,
                firmware::BASIC_SRC,
            );
            mem.map_rom(
                ROM_LAYER,
                firmware::KC87_OS_DEST,
                firmware::KC87_ROM_BANK_SIZE,
                firmware::KC87_OS_SRC,
            );
        }
        mem.map_ram(
            RAM_LAYER,
            memory_map::VIDEO_RAM_START,
            memory_map::VIDEO_RAM_SIZE as u32,
            memory_map::VIDEO_RAM_START as usize,
        );

        Self {
            ram,
            rom,
            mem,
            pio1: U855::new(),
            pio2: U855::new(),
            ctc: U857::new(),
            rtc: if hardware.rtc {
                Some(Rtc7242x::new())
            } else {
                None
            },
            fdc: if hardware.floppy {
                let mut fdc = U8272::new(FDC_MHZ);
                fdc.set_tstates_per_milli(MASTER_CLOCK_HZ / MILLIS_PER_SECOND);
                Some(fdc)
            } else {
                None
            },
            fdc_terminal_count: false,
            fdc_reset_line: false,
            keyboard,
            user_slot: UserPeripheral::None,
            blink_flip_flop: 0,
            blink_counter: 0,
            ctc_zcto2: 0,
            sys_porta: 0,
            chargen: hardware.chargen,
            chargen_ram: vec![0u8; 0x400].into_boxed_slice().try_into().unwrap(),
            chargen_window: false,
            chargen_active: false,
            ram64k_module,
            ram_ext: vec![0u8; 0x4000].into_boxed_slice().try_into().unwrap(),
            ram_4000_seg1: false,
            ram_c000: false,
            c000_ram_end,
            rom_module,
            rom_module_end,
            has_overlays,
            graph_type: hardware.graphics,
            ram_pixel: vec![0u8; 0x2000].into_boxed_slice().try_into().unwrap(),
            graph_mode: false,
            graph_fg: 0,
            graph_bg: 0,
            graph_border: false,
            graph_bank: 0,
            graph_addr_l: 0,
            c80_enabled: hardware.c80,
            ram_video2: vec![0u8; 0x400].into_boxed_slice().try_into().unwrap(),
            ram_color2: vec![0u8; 0x400].into_boxed_slice().try_into().unwrap(),
            c80_active: false,
            c80_memswap: false,
            current_cycle: 0,
            prev_user_write: false,
            pio_ea: U855::new(),
            ea_present: false,
            ea_slot: UserPeripheral::None,
            prev_ea_write: false,
            line_tstates: 0,
            line_num: 0,
            wait_pending: 0,
            tape_rx: None,
            tape_sample_rate: 0,
            tape_tract: TapeTract::new(),
            tape_acc: 0,
        }
    }

    pub fn attach_tape(&mut self, rx: Receiver<f32>, sample_rate: u32) {
        self.tape_rx = Some(rx);
        self.tape_sample_rate = sample_rate;
        self.tape_acc = 0;
        self.tape_tract = TapeTract::new();
    }

    pub fn insert_disk(&mut self, drive_num: usize, disk: Box<dyn DiskBackend>) {
        if let Some(fdc) = &mut self.fdc {
            fdc.insert_disk(drive_num, disk);
        }
    }

    pub fn enable_ea_module(&mut self) {
        self.ea_present = true;
    }

    #[inline]
    pub fn audio_enabled(&self) -> bool {
        (self.sys_porta & SYS_PORTA_AUDIO_BIT) != 0
    }

    #[inline]
    pub fn border_color(&self) -> u8 {
        (self.sys_porta >> SYS_PORTA_BORDER_SHIFT) & COLOR_INDEX_MASK
    }

    #[inline]
    pub fn mode_20_rows(&self) -> bool {
        (self.sys_porta & SYS_PORTA_MODE20_BIT) != 0
    }

    #[inline]
    pub fn graph_robotron_active(&self) -> bool {
        self.graph_mode && self.graph_type == GraphicsModule::Robotron
    }

    #[inline]
    pub fn graph_krt_active(&self) -> bool {
        self.graph_mode && self.graph_type == GraphicsModule::Krt
    }

    #[inline(always)]
    fn read_memory(&self, addr: u16) -> u8 {
        if self.has_overlays {
            if self.chargen
                && self.chargen_window
                && (memory_map::COLOR_RAM_START..memory_map::COLOR_RAM_END).contains(&addr)
            {
                return self.chargen_ram[(addr - memory_map::COLOR_RAM_START) as usize];
            }
            if self.graph_type == GraphicsModule::Krt
                && self.graph_mode
                && (memory_map::VIDEO_RAM_START..memory_map::VIDEO_RAM_END).contains(&addr)
            {
                return self.ram_pixel[(self.graph_bank as usize) * memory_map::VIDEO_RAM_SIZE
                    + (addr - memory_map::VIDEO_RAM_START) as usize];
            }
            if self.c80_enabled && self.c80_memswap {
                if (memory_map::COLOR_RAM_START..memory_map::COLOR_RAM_END).contains(&addr) {
                    return self.ram_color2[(addr - memory_map::COLOR_RAM_START) as usize];
                }
                if (memory_map::VIDEO_RAM_START..memory_map::VIDEO_RAM_END).contains(&addr) {
                    return self.ram_video2[(addr - memory_map::VIDEO_RAM_START) as usize];
                }
            }
            if self.ram64k_module {
                if self.ram_4000_seg1
                    && (memory_map::RAM_EXT_SEG1_START..memory_map::RAM_EXT_SEG1_END)
                        .contains(&addr)
                {
                    return self.ram_ext[(addr - memory_map::RAM_EXT_SEG1_START) as usize];
                }
                if self.ram_c000 && (memory_map::UPPER_RAM_START..self.c000_ram_end).contains(&addr)
                {
                    return self.ram[addr as usize];
                }
            }
            if let Some(module) = &self.rom_module
                && (memory_map::UPPER_RAM_START..self.rom_module_end).contains(&addr)
            {
                return module[(addr - memory_map::UPPER_RAM_START) as usize];
            }
        }
        let page = &self.mem.page_table[(addr >> PAGE_SHIFT) as usize];
        let offset = (addr & PAGE_OFFSET_MASK) as usize;
        match page.read {
            PageRef::Ram(base) => self.ram[base + offset],
            PageRef::Rom(base) => self.rom[base + offset],
            PageRef::Unmapped => OPEN_BUS_VALUE,
        }
    }

    #[inline(always)]
    pub fn write_memory(&mut self, addr: u16, data: u8) {
        if self.has_overlays {
            if self.chargen {
                match addr {
                    memory_map::CHARGEN_WINDOW_ON => {
                        self.chargen_window = true;
                        self.chargen_active = false;
                    }
                    memory_map::CHARGEN_FONT_ON => {
                        self.chargen_active = true;
                        self.chargen_window = false;
                    }
                    memory_map::CHARGEN_OFF => {
                        self.chargen_active = false;
                        self.chargen_window = false;
                    }
                    _ => {}
                }
                if self.chargen_window
                    && (memory_map::COLOR_RAM_START..memory_map::COLOR_RAM_END).contains(&addr)
                {
                    self.chargen_ram[(addr - memory_map::COLOR_RAM_START) as usize] = data;
                    return;
                }
            }
            if self.ram64k_module {
                if self.ram_4000_seg1
                    && (memory_map::RAM_EXT_SEG1_START..memory_map::RAM_EXT_SEG1_END)
                        .contains(&addr)
                {
                    self.ram_ext[(addr - memory_map::RAM_EXT_SEG1_START) as usize] = data;
                    return;
                }
                if self.ram_c000 && (memory_map::UPPER_RAM_START..self.c000_ram_end).contains(&addr)
                {
                    self.ram[addr as usize] = data;
                    return;
                }
            }
            if self.graph_type == GraphicsModule::Krt
                && self.graph_mode
                && (memory_map::VIDEO_RAM_START..memory_map::VIDEO_RAM_END).contains(&addr)
            {
                self.ram_pixel[(self.graph_bank as usize) * memory_map::VIDEO_RAM_SIZE
                    + (addr - memory_map::VIDEO_RAM_START) as usize] = data;
                return;
            }
            if self.c80_enabled && self.c80_memswap {
                if (memory_map::COLOR_RAM_START..memory_map::COLOR_RAM_END).contains(&addr) {
                    self.ram_color2[(addr - memory_map::COLOR_RAM_START) as usize] = data;
                    return;
                }
                if (memory_map::VIDEO_RAM_START..memory_map::VIDEO_RAM_END).contains(&addr) {
                    self.ram_video2[(addr - memory_map::VIDEO_RAM_START) as usize] = data;
                    return;
                }
            }
        }
        let page = &self.mem.page_table[(addr >> PAGE_SHIFT) as usize];
        let offset = (addr & PAGE_OFFSET_MASK) as usize;
        match page.write {
            PageRef::Ram(base) => self.ram[base + offset] = data,
            PageRef::Rom(base) => self.rom[base + offset] = data,
            PageRef::Unmapped => {}
        }
    }

    #[inline(always)]
    fn is_io_device(pins: u64, expected_a5_a4_a3: u64) -> bool {
        let mask = pins::IORQ | pins::M1 | pins::A7 | pins::A6 | pins::A5 | pins::A4 | pins::A3;
        let expected = pins::IORQ | pins::A7 | expected_a5_a4_a3;
        (pins & mask) == expected
    }

    #[inline(always)]
    fn is_contended(&self, addr: u16) -> bool {
        if (memory_map::VIDEO_RAM_START..memory_map::VIDEO_RAM_END).contains(&addr) {
            return true;
        }
        if (memory_map::COLOR_RAM_START..memory_map::COLOR_RAM_END).contains(&addr) {
            return !(self.chargen && self.chargen_window);
        }
        false
    }

    #[inline(always)]
    pub fn tick(&mut self, mut pins: u64) -> (u64, bool) {
        let mut waiting = false;
        if (pins & pins::MREQ) != 0 {
            let addr = pins::addr(pins);
            if (pins & pins::RD) != 0 {
                pins = pins::set_data(pins, self.read_memory(addr));
            } else if (pins & pins::WR) != 0 {
                self.write_memory(addr, pins::data(pins));
            }
            if (pins & (pins::RD | pins::WR)) != 0
                && self.line_num < VIDEO_VISIBLE_LINES
                && self.line_tstates < TSTATES_VISIBLE
                && self.is_contended(addr)
            {
                self.wait_pending = TSTATES_VISIBLE - self.line_tstates;
                waiting = true;
            }
        }
        if waiting {
            pins |= pins::WAIT;
        } else if self.wait_pending > 0 {
            pins |= pins::WAIT;
            self.wait_pending -= 1;
        } else {
            pins &= !pins::WAIT;
        }

        pins |= pins::IEIO;
        if Self::is_io_device(pins, pins::A4) {
            pins |= u855::CE;
        }
        if (pins & pins::A0) != 0 {
            pins |= u855::BASEL;
        }
        if (pins & pins::A1) != 0 {
            pins |= u855::CDSEL;
        }

        let pa_in = (!self.keyboard.scan_columns()) as u8;
        let pb_in = (!self.keyboard.scan_lines()) as u8;
        pins = (pins & !(0xFFFF_u64 << PIO_PORT_A_SHIFT))
            | ((pa_in as u64) << PIO_PORT_A_SHIFT)
            | ((pb_in as u64) << PIO_PORT_B_SHIFT);

        pins = self.pio2.tick(pins);

        let pa_out = !((pins >> PIO_PORT_A_SHIFT) as u8);
        let pb_out = !((pins >> PIO_PORT_B_SHIFT) as u8);
        self.keyboard.set_active_columns(pa_out as u16);
        self.keyboard.set_active_lines(pb_out as u16);
        pins &= pins::PIN_MASK;

        if Self::is_io_device(pins, pins::A3) {
            pins |= u855::CE;
        }
        if (pins & pins::A0) != 0 {
            pins |= u855::BASEL;
        }
        if (pins & pins::A1) != 0 {
            pins |= u855::CDSEL;
        }

        let user_write = (pins & (u855::CE | pins::IORQ | pins::M1 | pins::RD))
            == (u855::CE | pins::IORQ)
            && (pins & u855::BASEL) != 0
            && (pins & u855::CDSEL) == 0;
        // Woran merkt man, dass die Stasi Robotron-Wanzen bei einem einsetzt?
        if user_write
            && !self.prev_user_write
            && let UserPeripheral::Midi(midi) = &mut self.user_slot
        {
            midi.push_byte(pins::data(pins), self.current_cycle);
        }
        self.prev_user_write = user_write;

        if self.ram64k_module && (pins & (pins::IORQ | pins::M1)) == pins::IORQ {
            match (pins::addr(pins) & PORT_ADDR_MASK) as u8 {
                ports::RAM64K_SEG1_OFF => self.ram_4000_seg1 = false,
                ports::RAM64K_SEG1_ON => self.ram_4000_seg1 = true,
                ports::RAM64K_C000_OFF => self.ram_c000 = false,
                ports::RAM64K_C000_ON => self.ram_c000 = true,
                _ => {}
            }
        }

        if self.c80_enabled && (pins & (pins::IORQ | pins::M1 | pins::RD)) == pins::IORQ {
            match (pins::addr(pins) & PORT_ADDR_MASK) as u8 {
                ports::C80_SWAP_OFF if self.fdc.is_none() => self.c80_memswap = false,
                ports::C80_SWAP_ON if self.fdc.is_none() => self.c80_memswap = true,
                ports::C80_SWAP_OFF_ALT => self.c80_memswap = false,
                ports::C80_SWAP_ON_ALT => self.c80_memswap = true,
                ports::C80_WIDE_OFF | ports::C80_WIDE_OFF_ALT => self.c80_active = false,
                ports::C80_WIDE_ON | ports::C80_WIDE_ON_ALT => self.c80_active = true,
                _ => {}
            }
        }

        let mut tape_strobe = false;
        if let Some(rx) = &self.tape_rx {
            self.tape_acc = self.tape_acc.saturating_add(self.tape_sample_rate);
            while self.tape_acc >= MASTER_CLOCK_HZ {
                match rx.try_recv() {
                    Ok(sample) => {
                        self.tape_acc -= MASTER_CLOCK_HZ;
                        if self.tape_tract.push(sample).is_some() {
                            tape_strobe = true;
                        }
                    }
                    Err(_) => {
                        self.tape_acc = MASTER_CLOCK_HZ;
                        break;
                    }
                }
            }
        }
        if tape_strobe {
            pins |= u855::ASTB;
        }

        pins = self.pio1.tick(pins);
        self.sys_porta = self.pio1.output_a();
        pins &= pins::PIN_MASK;

        if self.ea_present {
            if (pins & (pins::IORQ | pins::M1)) == pins::IORQ
                && ((pins::addr(pins) & PORT_ADDR_MASK) as u8 & ports::EA_PORT_MASK)
                    == ports::EA_BASE_C8
            {
                pins |= u855::CE;
                if (pins & pins::A0) != 0 {
                    pins |= u855::BASEL;
                }
                if (pins & pins::A1) != 0 {
                    pins |= u855::CDSEL;
                }
            }
            let ea_write = (pins & (u855::CE | pins::IORQ | pins::M1 | pins::RD))
                == (u855::CE | pins::IORQ)
                && (pins & u855::BASEL) != 0
                && (pins & u855::CDSEL) == 0;
            if ea_write
                && !self.prev_ea_write
                && let UserPeripheral::Midi(midi) = &mut self.ea_slot
            {
                midi.push_byte(pins::data(pins), self.current_cycle);
            }
            self.prev_ea_write = ea_write;
            pins = self.pio_ea.tick(pins);
            pins &= pins::PIN_MASK;
        }

        pins |= self.ctc_zcto2;
        if Self::is_io_device(pins, 0) {
            pins |= u857::CE;
        }
        if (pins & pins::A0) != 0 {
            pins |= u857::CS0;
        }
        if (pins & pins::A1) != 0 {
            pins |= u857::CS1;
        }
        if (pins & u857::ZCTO2) != 0 {
            pins |= u857::CLKTRG3;
        }

        pins = self.ctc.tick(pins);
        let beeper_toggled = (pins & u857::ZCTO0) != 0;
        self.ctc_zcto2 = pins & u857::ZCTO2;
        pins &= pins::PIN_MASK;

        if let Some(rtc) = &mut self.rtc
            && (pins & (pins::IORQ | pins::M1)) == pins::IORQ
            && (pins::addr(pins) as u8 & ports::RTC_BASE_MASK) == ports::RTC_BASE
        {
            let reg = pins::addr(pins) as u8 & ports::RTC_REG_MASK;
            if (pins & pins::RD) != 0 {
                pins = pins::set_data(pins, rtc.read(reg));
            } else {
                rtc.write(reg, pins::data(pins));
            }
        }

        if let Some(fdc) = &mut self.fdc
            && (pins & (pins::IORQ | pins::M1)) == pins::IORQ
        {
            let port = (pins::addr(pins) & PORT_ADDR_MASK) as u8;
            let reading = (pins & pins::RD) != 0;
            if (port & ports::FDC_PORT_MASK) == ports::FDC_BASE {
                if (port & ports::FDC_DATA_SELECT) != 0 {
                    if reading {
                        pins = pins::set_data(pins, fdc.read_data());
                    } else {
                        fdc.write(pins::data(pins));
                    }
                } else if reading {
                    pins = pins::set_data(pins, fdc.read_main_status_reg());
                }
            } else if !reading && (port & ports::FDC_PORT_MASK) == ports::FDC_CONTROL_BASE {
                let value = pins::data(pins);
                let terminal_count = (value & ports::FDC_CONTROL_TC) != 0;
                if terminal_count && !self.fdc_terminal_count {
                    fdc.fire_tc();
                }
                let reset_line = (value & ports::FDC_CONTROL_RESET) != 0;
                if reset_line && !self.fdc_reset_line {
                    fdc.reset(false);
                }
                self.fdc_terminal_count = terminal_count;
                self.fdc_reset_line = reset_line;
            }
        }

        if let Some(fdc) = &mut self.fdc {
            fdc.tick();
        }

        if self.graph_type != GraphicsModule::None && (pins & (pins::IORQ | pins::M1)) == pins::IORQ
        {
            let addr = pins::addr(pins);
            let reading = (pins & pins::RD) != 0;
            match (addr & PORT_ADDR_MASK) as u8 {
                ports::GRAPH_CONTROL => {
                    if reading {
                        let mut v = match self.graph_type {
                            GraphicsModule::Robotron => {
                                self.graph_bg | (self.graph_fg << COLOR_NIBBLE_SHIFT)
                            }
                            GraphicsModule::Krt => self.graph_bank,
                            GraphicsModule::None => 0,
                        };
                        if self.graph_mode {
                            v |= GRAPH_MODE_BIT;
                        }
                        if self.graph_type == GraphicsModule::Robotron && self.graph_border {
                            v |= GRAPH_BORDER_READ_BIT;
                        }
                        pins = pins::set_data(pins, v);
                    } else {
                        let v = pins::data(pins);
                        match self.graph_type {
                            GraphicsModule::Robotron => {
                                self.graph_bg = v & COLOR_INDEX_MASK;
                                self.graph_fg = (v >> COLOR_NIBBLE_SHIFT) & COLOR_INDEX_MASK;
                                self.graph_border = (v & GRAPH_BORDER_WRITE_BIT) != 0;
                                self.graph_mode = (v & GRAPH_MODE_BIT) != 0;
                            }
                            GraphicsModule::Krt => {
                                self.graph_bank = v & COLOR_INDEX_MASK;
                                self.graph_mode = (v & GRAPH_MODE_BIT) != 0;
                            }
                            GraphicsModule::None => {}
                        }
                    }
                }
                ports::GRAPH_ADDR_LOW
                    if self.graph_type == GraphicsModule::Robotron && !reading =>
                {
                    self.graph_addr_l = pins::data(pins);
                }
                ports::GRAPH_PIXEL if self.graph_type == GraphicsModule::Robotron => {
                    let px = (((addr >> 8) as usize) << 8) | self.graph_addr_l as usize;
                    if px < self.ram_pixel.len() {
                        if reading {
                            pins = pins::set_data(pins, self.ram_pixel[px]);
                        } else {
                            self.ram_pixel[px] = pins::data(pins);
                        }
                    }
                }
                _ => {}
            }
        }

        let old_blink = self.blink_counter;
        self.blink_counter = self.blink_counter.wrapping_sub(1);
        if old_blink == 0 {
            self.blink_counter = BLINK_TOGGLE_CYCLES;
            self.blink_flip_flop ^= BLINK_FLIP_FLOP_BIT;
        }

        self.line_tstates += 1;
        if self.line_tstates >= TSTATES_PER_LINE {
            self.line_tstates -= TSTATES_PER_LINE;
            self.line_num += 1;
            if self.line_num >= VIDEO_TOTAL_LINES {
                self.line_num = 0;
            }
        }

        (pins, beeper_toggled)
    }
}
