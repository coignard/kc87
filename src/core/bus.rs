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

use crate::core::chips::u855::{self, U855};
use crate::core::chips::u857::{self, U857};
use crate::core::machine::{GraphicsModule, Hardware, MachineType, RamSize};
use crate::core::peripherals::UserPeripheral;
use crate::core::peripherals::keyboard::Keyboard;

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
    pub page_table: [MemPage; 64],
    pub layers: [[MemPage; 64]; 4],
}

impl Default for Mem {
    fn default() -> Self {
        Self::new()
    }
}

impl Mem {
    pub fn new() -> Self {
        Self {
            page_table: [MemPage::default(); 64],
            layers: [[MemPage::default(); 64]; 4],
        }
    }

    pub fn update_page_table(&mut self, page_index: usize) {
        for layer in 0..4 {
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
        let num_pages = (size >> 10) as usize;
        for i in 0..num_pages {
            let offset = i * 1024;
            let page_index = (((addr as usize) + offset) & 0xFFFF) >> 10;

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
    pub current_cycle: u64,
    #[serde(skip)]
    prev_user_write: bool,
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

        let mut keyboard = Keyboard::new(3);
        keyboard.register_modifier(0, 0, 7);
        let keymap: &[u8] = b"0123456789:;,=.?@ABCDEFGHIJKLMNOPQRSTUVWXYZ   ^                 _!\"#$%&'()*+<->/ abcdefghijklmnopqrstuvwxyz                     ";
        for shift in 0..2 {
            for line in 0..8 {
                for column in 0..8 {
                    let c = keymap[shift * 64 + line * 8 + column];
                    if c != b' ' {
                        keyboard.register_key(
                            c as i32,
                            column,
                            line,
                            if shift != 0 { 1 } else { 0 },
                        );
                    }
                }
            }
        }
        keyboard.register_key(0x03, 6, 6, 0);
        keyboard.register_key(0x08, 0, 6, 0);
        keyboard.register_key(0x09, 1, 6, 0);
        keyboard.register_key(0x0A, 2, 6, 0);
        keyboard.register_key(0x0B, 3, 6, 0);
        keyboard.register_key(0x0D, 5, 6, 0);
        keyboard.register_key(0x13, 4, 5, 0);
        keyboard.register_key(0x14, 1, 7, 0);
        keyboard.register_key(0x19, 3, 5, 0);
        keyboard.register_key(0x1A, 5, 5, 0);
        keyboard.register_key(0x1B, 4, 6, 0);
        keyboard.register_key(0x1C, 4, 7, 0);
        keyboard.register_key(0x1D, 5, 7, 0);
        keyboard.register_key(0x20, 7, 6, 0);

        let ram_top: u32 = match hardware.ram {
            RamSize::K16 => 0x4000,
            RamSize::K32 => 0x8000,
            RamSize::K48 | RamSize::K64 => 0xC000,
            RamSize::Default => match machine_type {
                MachineType::Z9001 => 0x8000,
                MachineType::KC87 => 0xC000,
            },
        };

        let ram64k_module = hardware.ram == RamSize::K64;
        let c000_ram_end: u16 = 0xE800;

        let (rom_module, rom_module_end) = match rom_module {
            Some(bytes) if !bytes.is_empty() => {
                let end = (0xC000u32 + bytes.len() as u32).min(c000_ram_end as u32) as u16;
                (Some(bytes.into_boxed_slice()), end)
            }
            _ => (None, 0xC000),
        };

        let has_overlays = hardware.chargen
            || ram64k_module
            || rom_module.is_some()
            || hardware.graphics != GraphicsModule::None;

        if machine_type == MachineType::Z9001 {
            if !basic_rom.is_empty() {
                load_rom(&mut rom, 0x0000, &basic_rom);
                mem.map_rom(1, 0xC000, 0x2800, 0x0000);
            }
            load_rom(&mut rom, 0x3000, &os_rom_1);
            mem.map_rom(1, 0xF000, 0x0800, 0x3000);
            if let Some(os_2) = os_rom_2 {
                load_rom(&mut rom, 0x3800, &os_2);
                mem.map_rom(1, 0xF800, 0x0800, 0x3800);
            }
            mem.map_ram(0, 0x0000, ram_top, 0x0000);
        } else {
            load_rom(&mut rom, 0x0000, &basic_rom);
            load_rom(&mut rom, 0x2000, &os_rom_1);
            mem.map_ram(0, 0x0000, ram_top, 0x0000);
            mem.map_ram(0, 0xE800, 0x0400, 0xE800);
            mem.map_rom(1, 0xC000, 0x2000, 0x0000);
            mem.map_rom(1, 0xE000, 0x2000, 0x2000);
        }
        mem.map_ram(0, 0xEC00, 0x0400, 0xEC00);

        Self {
            ram,
            rom,
            mem,
            pio1: U855::new(),
            pio2: U855::new(),
            ctc: U857::new(),
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
            current_cycle: 0,
            prev_user_write: false,
        }
    }

    #[inline]
    pub fn audio_enabled(&self) -> bool {
        (self.sys_porta & 0x80) != 0
    }

    #[inline]
    pub fn border_color(&self) -> u8 {
        (self.sys_porta >> 3) & 0x07
    }

    #[inline]
    pub fn mode_20_rows(&self) -> bool {
        (self.sys_porta & 0x04) != 0
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
            if self.chargen && self.chargen_window && (0xE800..0xEC00).contains(&addr) {
                return self.chargen_ram[(addr - 0xE800) as usize];
            }
            if self.graph_type == GraphicsModule::Krt
                && self.graph_mode
                && (0xEC00..0xF000).contains(&addr)
            {
                return self.ram_pixel
                    [(self.graph_bank as usize) * 0x400 + (addr - 0xEC00) as usize];
            }
            if self.ram64k_module {
                if self.ram_4000_seg1 && (0x4000..0x8000).contains(&addr) {
                    return self.ram_ext[(addr - 0x4000) as usize];
                }
                if self.ram_c000 && (0xC000..self.c000_ram_end).contains(&addr) {
                    return self.ram[addr as usize];
                }
            }
            if let Some(module) = &self.rom_module
                && (0xC000..self.rom_module_end).contains(&addr)
            {
                return module[(addr - 0xC000) as usize];
            }
        }
        let page = &self.mem.page_table[(addr >> 10) as usize];
        let offset = (addr & 1023) as usize;
        match page.read {
            PageRef::Ram(base) => self.ram[base + offset],
            PageRef::Rom(base) => self.rom[base + offset],
            PageRef::Unmapped => 0xFF,
        }
    }

    #[inline(always)]
    pub fn write_memory(&mut self, addr: u16, data: u8) {
        if self.has_overlays {
            if self.chargen {
                match addr {
                    0xEBFC => {
                        self.chargen_window = true;
                        self.chargen_active = false;
                    }
                    0xEBFE => {
                        self.chargen_active = true;
                        self.chargen_window = false;
                    }
                    0xEBFF => {
                        self.chargen_active = false;
                        self.chargen_window = false;
                    }
                    _ => {}
                }
                if self.chargen_window && (0xE800..0xEC00).contains(&addr) {
                    self.chargen_ram[(addr - 0xE800) as usize] = data;
                    return;
                }
            }
            if self.ram64k_module {
                if self.ram_4000_seg1 && (0x4000..0x8000).contains(&addr) {
                    self.ram_ext[(addr - 0x4000) as usize] = data;
                    return;
                }
                if self.ram_c000 && (0xC000..self.c000_ram_end).contains(&addr) {
                    self.ram[addr as usize] = data;
                    return;
                }
            }
            if self.graph_type == GraphicsModule::Krt
                && self.graph_mode
                && (0xEC00..0xF000).contains(&addr)
            {
                self.ram_pixel[(self.graph_bank as usize) * 0x400 + (addr - 0xEC00) as usize] =
                    data;
                return;
            }
        }
        let page = &self.mem.page_table[(addr >> 10) as usize];
        let offset = (addr & 1023) as usize;
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
    pub fn tick(&mut self, mut pins: u64) -> (u64, bool) {
        if (pins & pins::MREQ) != 0 {
            let addr = pins::addr(pins);
            if (pins & pins::RD) != 0 {
                pins = pins::set_data(pins, self.read_memory(addr));
            } else if (pins & pins::WR) != 0 {
                self.write_memory(addr, pins::data(pins));
            }
        }

        pins |= pins::IEIO;
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
        if user_write
            && !self.prev_user_write
            && let UserPeripheral::Midi(midi) = &mut self.user_slot
        {
            midi.push_byte(pins::data(pins), self.current_cycle);
        }
        self.prev_user_write = user_write;

        if self.ram64k_module && (pins & (pins::IORQ | pins::M1 | pins::RD)) == pins::IORQ {
            match (pins::addr(pins) & 0xFF) as u8 {
                0x04 => self.ram_4000_seg1 = false,
                0x05 => self.ram_4000_seg1 = true,
                0x06 => self.ram_c000 = false,
                0x07 => self.ram_c000 = true,
                _ => {}
            }
        }

        pins = self.pio1.tick(pins);
        self.sys_porta = self.pio1.output_a();
        pins &= pins::PIN_MASK;

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
        pins = (pins & !(0xFFFF_u64 << 48)) | ((pa_in as u64) << 48) | ((pb_in as u64) << 56);

        pins = self.pio2.tick(pins);

        let pa_out = !((pins >> 48) as u8);
        let pb_out = !((pins >> 56) as u8);
        self.keyboard.set_active_columns(pa_out as u16);
        self.keyboard.set_active_lines(pb_out as u16);
        pins &= pins::PIN_MASK;

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

        if self.graph_type != GraphicsModule::None && (pins & (pins::IORQ | pins::M1)) == pins::IORQ
        {
            let addr = pins::addr(pins);
            let reading = (pins & pins::RD) != 0;
            match (addr & 0xFF) as u8 {
                0xB8 => {
                    if reading {
                        let mut v = match self.graph_type {
                            GraphicsModule::Robotron => self.graph_bg | (self.graph_fg << 4),
                            GraphicsModule::Krt => self.graph_bank,
                            GraphicsModule::None => 0,
                        };
                        if self.graph_mode {
                            v |= 0x08;
                        }
                        if self.graph_type == GraphicsModule::Robotron && self.graph_border {
                            v |= 0x40;
                        }
                        pins = pins::set_data(pins, v);
                    } else {
                        let v = pins::data(pins);
                        match self.graph_type {
                            GraphicsModule::Robotron => {
                                self.graph_bg = v & 0x07;
                                self.graph_fg = (v >> 4) & 0x07;
                                self.graph_border = (v & 0x80) != 0;
                                self.graph_mode = (v & 0x08) != 0;
                            }
                            GraphicsModule::Krt => {
                                self.graph_bank = v & 0x07;
                                self.graph_mode = (v & 0x08) != 0;
                            }
                            GraphicsModule::None => {}
                        }
                    }
                }
                0xB9 if self.graph_type == GraphicsModule::Robotron && !reading => {
                    self.graph_addr_l = pins::data(pins);
                }
                0xBA if self.graph_type == GraphicsModule::Robotron => {
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
            self.blink_counter = (2_457_600 * 8) / 25;
            self.blink_flip_flop ^= 0x80;
        }

        (pins, beeper_toggled)
    }
}
