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

use crate::core::peripherals::disk::{FloppyDisk, FloppyDiskDrive, Sector, SectorReader};

pub const NUM_DRIVES: usize = 4;

const ARG_COUNT: usize = 9;
const RESULT_COUNT: usize = 7;

const ARG_COMMAND: usize = 0;
const ARG_HEAD_DRIVE: usize = 1;
const ARG_CYLINDER: usize = 2;
const ARG_HEAD: usize = 3;
const ARG_RECORD: usize = 4;
const ARG_SIZE_CODE: usize = 5;
const ARG_END_OF_TRACK: usize = 6;
const ARG_DATA_LENGTH: usize = 8;
const ARG_SCAN_STEP: usize = 8;

const ARG_FORMAT_SIZE_CODE: usize = 2;
const ARG_FORMAT_SECTOR_COUNT: usize = 3;
const ARG_FORMAT_FILLER: usize = 5;

const ARG_PHASE_COMMAND: usize = 0;
const ARG_PHASE_READ_WRITE_SCAN: usize = 9;
const ARG_PHASE_READ_ID: usize = 2;
const ARG_PHASE_FORMAT: usize = 6;
const ARG_PHASE_RECALIBRATE: usize = 2;
const ARG_PHASE_SEEK: usize = 3;
const ARG_PHASE_SENSE_DRIVE: usize = 2;
const ARG_PHASE_SPECIFY: usize = 3;

const FORMAT_ID_FIELD_LEN: usize = 4;
const FORMAT_ID_CYLINDER_OFFSET: usize = 0;
const FORMAT_ID_HEAD_OFFSET: usize = 1;
const FORMAT_ID_RECORD_OFFSET: usize = 2;
const FORMAT_ID_SIZE_CODE_OFFSET: usize = 3;

const ARG0_SK_MASK: u8 = 0x20;
const ARG0_MT_MASK: u8 = 0x80;
const DRIVE_MASK: u8 = 0x03;
const HEAD_MASK: u8 = 0x04;
const HEAD_DRIVE_MASK: u8 = HEAD_MASK | DRIVE_MASK;
const HEAD_SHIFT: u8 = 2;
const HEAD_SELECT_MASK: u8 = HEAD_MASK >> HEAD_SHIFT;

const COMMAND_MASK: u8 = 0x1F;
const SIZE_CODE_MASK: u8 = 0x0F;
const SECTOR_SIZE_BASE: usize = 128;

const STM_REQUEST_FOR_MASTER: u8 = 0x80;
const STM_DATA_INPUT: u8 = 0x40;
const STM_NON_DMA_MODE: u8 = 0x20;
const STM_BUSY: u8 = 0x10;
const STM_DRIVE_MASK: u8 = 0x0F;

const ST0_ERROR_MASK: u8 = 0xC0;
const ST0_ABORT_BECAUSE_READY_CHANGED: u8 = 0xC0;
const ST0_INVALID_COMMAND_ISSUE: u8 = 0x80;
const ST0_ABNORMAL_TERMINATION: u8 = 0x40;
const ST0_SEEK_END: u8 = 0x20;
const ST0_EQUIPMENT_CHECK: u8 = 0x10;
const ST0_NOT_READY: u8 = 0x08;
const ST0_SEEK_FLAGS_MASK: u8 = 0xF8;

const ST1_END_OF_CYLINDER: u8 = 0x80;
const ST1_DATA_ERROR: u8 = 0x20;
const ST1_OVERRUN: u8 = 0x10;
const ST1_NO_DATA: u8 = 0x04;
const ST1_NOT_WRITABLE: u8 = 0x02;
const ST1_MISSING_ADDRESS_MARK: u8 = 0x01;

const ST2_CONTROL_MARK: u8 = 0x40;
const ST2_DATA_ERROR_IN_DATA_FIELD: u8 = 0x20;
const ST2_SCAN_EQUAL_HIT: u8 = 0x08;
const ST2_SCAN_NOT_SATISFIED: u8 = 0x04;

const ST3_WRITE_PROTECTED: u8 = 0x40;
const ST3_READY: u8 = 0x20;
const ST3_TRACK_0: u8 = 0x10;
const ST3_TWO_SIDE: u8 = 0x08;

const SPECIFY_STEP_RATE_SHIFT: u8 = 4;
const SPECIFY_STEP_RATE_MASK: u8 = 0x0F;
const SPECIFY_STEP_RATE_BASE: u32 = 16;
const SPECIFY_NON_DMA_BIT: u8 = 0x01;

const TRACKS_PER_DISK: i32 = 77;
const ROTATIONS_PER_SECOND: u32 = 50;
const MILLIS_PER_SECOND: u32 = 1000;
const INDEX_HOLE_FRACTION: i32 = 100;
const IO_REQ_MILLI_DIVISOR: i32 = 100;
const IO_REQ_DELAY_CAP: i32 = 1;
const NEXT_SECTOR_ROTATION_DIVISOR: i32 = 5;
const HD_SPEED_THRESHOLD_KHZ: u32 = 5000;

const RESULT_IDLE: i32 = -1;
const DATA_POS_IDLE: i32 = -1;
const SEEK_STATUS_IDLE: i16 = -1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Command {
    FormatTrack,
    ReadData,
    ReadDeletedData,
    ReadId,
    ReadTrack,
    Recalibrate,
    ScanEqual,
    ScanLowOrEqual,
    ScanHighOrEqual,
    Seek,
    SenseDriveStatus,
    SenseInterruptStatus,
    Specify,
    WriteData,
    WriteDeletedData,
    Invalid,
}

const COMMAND_TABLE: [Command; 32] = [
    Command::Invalid,
    Command::Invalid,
    Command::ReadTrack,
    Command::Specify,
    Command::SenseDriveStatus,
    Command::WriteData,
    Command::ReadData,
    Command::Recalibrate,
    Command::SenseInterruptStatus,
    Command::WriteDeletedData,
    Command::ReadId,
    Command::Invalid,
    Command::ReadDeletedData,
    Command::FormatTrack,
    Command::Invalid,
    Command::Seek,
    Command::Invalid,
    Command::ScanEqual,
    Command::Invalid,
    Command::Invalid,
    Command::Invalid,
    Command::Invalid,
    Command::Invalid,
    Command::Invalid,
    Command::Invalid,
    Command::ScanLowOrEqual,
    Command::Invalid,
    Command::Invalid,
    Command::Invalid,
    Command::ScanHighOrEqual,
    Command::Invalid,
    Command::Invalid,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum FormatStatus {
    Idle,
    WaitForHole,
    ReceiveData,
    Busy,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IoTask {
    Idle,
    FormatTrack,
    ReadSectorById,
    ReadSectorByIndex,
    ReadSectorForWrite,
    WriteSector,
}

pub struct U8272 {
    drives: [FloppyDiskDrive; NUM_DRIVES],
    executing_drive: Option<usize>,
    cur_cmd: Command,
    format_status: FormatStatus,
    io_task_cmd: IoTask,
    tc_enabled: bool,
    tc_fired: bool,
    interrupt_req: bool,
    dma_req: bool,
    dma_mode: bool,
    hd_mode: bool,
    hd_possible: bool,
    seek_mode: bool,
    eot_reached: bool,
    args: [u8; ARG_COUNT],
    arg_idx: usize,
    results: [u8; RESULT_COUNT],
    result_idx: i32,
    sector_id_cyl: u8,
    sector_id_head: u8,
    sector_id_rec: u8,
    sector_id_size_code: u8,
    mhz: u32,
    status_reg_main: u8,
    status_reg0: u8,
    status_reg1: u8,
    status_reg2: u8,
    status_reg3: u8,
    step_rate_millis: u32,
    tstates_till_io_req: i32,
    tstates_till_io_start: i32,
    tstates_till_overrun: i32,
    tstate_rotation_counter: i32,
    tstate_step_counter: i32,
    tstates_per_milli: u32,
    tstates_per_rotation: i32,
    tstates_per_step: i32,
    seek_status: [i16; NUM_DRIVES],
    remain_seek_steps: [i32; NUM_DRIVES],
    data_buf: Vec<u8>,
    data_pos: i32,
    data_len: usize,
    remain_bytes: i32,
    cur_sector: Option<Sector>,
    cur_sector_reader: Option<SectorReader>,
}

impl U8272 {
    pub fn new(mhz: u32) -> Self {
        let mut fdc = Self {
            drives: [
                FloppyDiskDrive::new(),
                FloppyDiskDrive::new(),
                FloppyDiskDrive::new(),
                FloppyDiskDrive::new(),
            ],
            executing_drive: None,
            cur_cmd: Command::Invalid,
            format_status: FormatStatus::Idle,
            io_task_cmd: IoTask::Idle,
            tc_enabled: false,
            tc_fired: false,
            interrupt_req: false,
            dma_req: false,
            dma_mode: false,
            hd_mode: false,
            hd_possible: false,
            seek_mode: false,
            eot_reached: false,
            args: [0; ARG_COUNT],
            arg_idx: 0,
            results: [0; RESULT_COUNT],
            result_idx: RESULT_IDLE,
            sector_id_cyl: 0,
            sector_id_head: 0,
            sector_id_rec: 0,
            sector_id_size_code: 0,
            mhz,
            status_reg_main: 0,
            status_reg0: 0,
            status_reg1: 0,
            status_reg2: 0,
            status_reg3: 0,
            step_rate_millis: SPECIFY_STEP_RATE_BASE,
            tstates_till_io_req: 0,
            tstates_till_io_start: 0,
            tstates_till_overrun: 0,
            tstate_rotation_counter: 0,
            tstate_step_counter: 0,
            tstates_per_milli: 0,
            tstates_per_rotation: 0,
            tstates_per_step: 0,
            seek_status: [SEEK_STATUS_IDLE; NUM_DRIVES],
            remain_seek_steps: [0; NUM_DRIVES],
            data_buf: Vec::new(),
            data_pos: DATA_POS_IDLE,
            data_len: 0,
            remain_bytes: 0,
            cur_sector: None,
            cur_sector_reader: None,
        };
        fdc.reset(true);
        fdc
    }

    pub fn insert_disk(&mut self, drive_num: usize, disk: FloppyDisk) {
        if let Some(drive) = self.drives.get_mut(drive_num) {
            drive.insert_disk(disk);
        }
    }

    pub fn index_hole_state(&self) -> bool {
        self.tstate_rotation_counter < (self.tstates_per_rotation / INDEX_HOLE_FRACTION)
    }

    pub fn is_dma_request(&self) -> bool {
        self.dma_req
    }

    pub fn is_interrupt_request(&self) -> bool {
        self.interrupt_req
    }

    pub fn set_tstates_per_milli(&mut self, tstates_per_milli: u32) {
        self.tstates_per_milli = tstates_per_milli;
        self.hd_possible = tstates_per_milli > HD_SPEED_THRESHOLD_KHZ;
        if !self.hd_possible {
            self.hd_mode = false;
        }
        self.tstates_per_rotation =
            (tstates_per_milli * MILLIS_PER_SECOND / ROTATIONS_PER_SECOND) as i32;
        self.calc_tstates_per_step();
    }

    pub fn reset(&mut self, power_on: bool) {
        if power_on {
            self.dma_mode = false;
            self.step_rate_millis = SPECIFY_STEP_RATE_BASE;
        }
        self.executing_drive = None;
        self.hd_possible = false;
        self.hd_mode = false;
        self.seek_mode = false;
        self.tc_enabled = false;
        self.tc_fired = false;
        self.dma_req = false;
        self.interrupt_req = false;
        self.status_reg_main = STM_REQUEST_FOR_MASTER;
        self.status_reg3 = 0;
        self.format_status = FormatStatus::Idle;
        self.io_task_cmd = IoTask::Idle;
        self.cur_sector = None;
        self.cur_sector_reader = None;
        self.data_pos = DATA_POS_IDLE;
        self.data_len = 0;
        self.remain_bytes = 0;
        self.tstates_till_io_req = 0;
        self.tstates_till_io_start = 0;
        self.tstates_till_overrun = 0;
        self.tstate_rotation_counter = 0;
        self.tstate_step_counter = 0;
        self.seek_status = [SEEK_STATUS_IDLE; NUM_DRIVES];
        self.clear_sector_id();
        self.clear_regs012();
        self.args = [0; ARG_COUNT];
        self.results = [0; RESULT_COUNT];
        self.remain_seek_steps = [0; NUM_DRIVES];
        self.set_idle();
    }

    pub fn fire_tc(&mut self) {
        self.tstates_till_io_req = 0;
        self.tstates_till_io_start = 0;
        self.tstates_till_overrun = 0;
        match self.cur_cmd {
            Command::FormatTrack => {
                self.status_reg_main &= !STM_REQUEST_FOR_MASTER;
                match self.format_status {
                    FormatStatus::WaitForHole => self.stop_execution(),
                    FormatStatus::ReceiveData => {}
                    _ => self.set_idle(),
                }
            }
            Command::ReadData
            | Command::ReadDeletedData
            | Command::ReadTrack
            | Command::ScanEqual
            | Command::ScanLowOrEqual
            | Command::ScanHighOrEqual => {
                if self.tc_enabled {
                    self.stop_execution();
                }
            }
            Command::WriteData | Command::WriteDeletedData if self.tc_enabled => {
                self.tc_fired = true;
                self.tc_enabled = false;
                self.dma_req = false;
                self.status_reg_main &= !STM_REQUEST_FOR_MASTER;
                self.status_reg_main &= !STM_DATA_INPUT;
                self.status_reg_main &= !STM_NON_DMA_MODE;
                self.io_task_cmd = IoTask::WriteSector;
                self.tstates_till_io_start = 0;
            }
            _ => {}
        }
    }

    pub fn read_main_status_reg(&mut self) -> u8 {
        let value = self.status_reg_main;
        self.interrupt_req = false;
        value
    }

    pub fn read_data(&mut self) -> u8 {
        let mut value: i16 = -1;
        if !self.dma_mode {
            value = self.read_from_disk();
        }
        if value < 0 {
            value = self.status_reg_main as i16;
            if self.result_idx >= 0 && (self.result_idx as usize) < self.results.len() {
                value = self.results[self.result_idx as usize] as i16;
                self.result_idx -= 1;
                if self.result_idx < 0 {
                    self.set_idle();
                }
            }
        }
        self.interrupt_req = false;
        value as u8
    }

    pub fn read_dma(&mut self) -> u8 {
        let mut value: i16 = -1;
        self.dma_req = false;
        if self.dma_mode {
            value = self.read_from_disk();
        }
        value as u8
    }

    pub fn write(&mut self, value: u8) {
        if !self.dma_mode && self.executing_drive.is_some() {
            self.write_to_drive(value);
        } else {
            self.write_cmd(value);
        }
    }

    pub fn write_dma(&mut self, value: u8) {
        self.dma_req = false;
        if self.dma_mode {
            self.write_to_drive(value);
        }
    }

    pub fn tick(&mut self) {
        if self.io_task_cmd != IoTask::Idle {
            if self.tstates_till_io_start > 0 {
                self.tstates_till_io_start -= 1;
            }
            if self.tstates_till_io_start <= 0 {
                if self.eot_reached {
                    self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                    self.status_reg1 |= ST1_END_OF_CYLINDER;
                    self.stop_execution();
                    self.eot_reached = false;
                } else {
                    self.run_io_task();
                }
            }
        }

        if self.tstates_till_io_req > 0 {
            self.tstates_till_io_req -= 1;
            if self.tstates_till_io_req <= 0 {
                match self.cur_cmd {
                    Command::FormatTrack => self.set_byte_writable(false),
                    Command::ReadData | Command::ReadDeletedData | Command::ReadTrack => {
                        self.set_byte_readable()
                    }
                    Command::ScanEqual
                    | Command::ScanLowOrEqual
                    | Command::ScanHighOrEqual
                    | Command::WriteData
                    | Command::WriteDeletedData => self.set_byte_writable(true),
                    _ => {}
                }
            }
        }

        self.tstate_rotation_counter += 1;
        if self.tstates_per_rotation > 0
            && self.tstate_rotation_counter >= self.tstates_per_rotation
        {
            self.tstate_rotation_counter = 0;
            if self.cur_cmd == Command::FormatTrack {
                match self.format_status {
                    FormatStatus::WaitForHole => {
                        self.format_status = FormatStatus::ReceiveData;
                        self.data_pos = 0;
                        self.set_byte_writable(false);
                    }
                    FormatStatus::ReceiveData => {
                        self.start_io_task(IoTask::FormatTrack, 0);
                        self.format_status = FormatStatus::Busy;
                    }
                    _ => {}
                }
            }
        }

        if self.seek_mode {
            self.tstate_step_counter += 1;
            if self.tstate_step_counter >= self.tstates_per_step {
                self.tstate_step_counter = 0;
                self.exec_seek_step();
            }
        }

        if self.tstates_till_overrun > 0 {
            self.tstates_till_overrun -= 1;
            if self.tstates_till_overrun <= 0 && self.executing_drive.is_some() {
                self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                self.status_reg1 |= ST1_OVERRUN;
            }
        }
    }

    fn run_io_task(&mut self) {
        let task = self.io_task_cmd;
        self.io_task_cmd = IoTask::Idle;
        match task {
            IoTask::FormatTrack => self.exec_io_format_track(),
            IoTask::ReadSectorById => self.exec_io_read_sector_by_id(),
            IoTask::ReadSectorByIndex => self.exec_io_read_sector_by_index(),
            IoTask::ReadSectorForWrite => self.exec_io_read_sector_for_write(),
            IoTask::WriteSector => self.exec_io_write_sector(),
            IoTask::Idle => {}
        }
    }

    fn add_sector_num(&mut self, value: u8) {
        if self.sector_id_rec == self.args[ARG_END_OF_TRACK] {
            if (self.args[ARG_COMMAND] & ARG0_MT_MASK) != 0 {
                if (self.args[ARG_HEAD_DRIVE] & HEAD_MASK) == 0 {
                    self.sector_id_rec = 1;
                    self.args[ARG_HEAD_DRIVE] |= HEAD_MASK;
                } else {
                    self.eot_reached = true;
                    self.sector_id_rec = 1;
                    self.sector_id_cyl = self.sector_id_cyl.wrapping_add(1);
                    self.args[ARG_HEAD_DRIVE] &= !HEAD_MASK;
                }
            } else {
                self.eot_reached = true;
                self.sector_id_rec = 1;
                self.sector_id_cyl = self.sector_id_cyl.wrapping_add(1);
            }
        } else {
            self.sector_id_rec = self.sector_id_rec.wrapping_add(value);
        }
    }

    fn inc_sector_num(&mut self) {
        self.add_sector_num(1);
    }

    fn calc_tstates_per_step(&mut self) {
        self.tstates_per_step = (self.step_rate_millis * self.tstates_per_milli / self.mhz) as i32;
    }

    fn clear_regs012(&mut self) {
        self.status_reg0 = 0;
        self.status_reg1 = 0;
        self.status_reg2 = 0;
    }

    fn clear_sector_id(&mut self) {
        self.sector_id_cyl = 0;
        self.sector_id_head = 0;
        self.sector_id_rec = 0;
        self.sector_id_size_code = 0;
        self.status_reg0 = 0;
        self.status_reg1 = 0;
        self.status_reg2 = 0;
    }

    fn arg_head(&self) -> usize {
        ((self.args[ARG_HEAD_DRIVE] >> HEAD_SHIFT) & HEAD_SELECT_MASK) as usize
    }

    fn arg_drive(&self) -> usize {
        (self.args[ARG_HEAD_DRIVE] & DRIVE_MASK) as usize
    }

    fn arg_data_len(&self) -> usize {
        let size_code = self.args[ARG_SIZE_CODE] & SIZE_CODE_MASK;
        if size_code > 0 {
            SECTOR_SIZE_BASE << size_code
        } else {
            self.args[ARG_DATA_LENGTH] as usize
        }
    }

    fn exec_io_format_track(&mut self) {
        let mut done = false;
        let mut src_idx = 0usize;
        let drive_idx = self.executing_drive;

        if let Some(idx) = drive_idx {
            let disk_ok = self.drives[idx]
                .disk()
                .map(|disk| disk.is_hd() == self.hd_mode)
                .unwrap_or(false);
            if disk_ok && !self.data_buf.is_empty() {
                let n_sectors = self.data_pos as usize / FORMAT_ID_FIELD_LEN;
                if n_sectors > 0 && n_sectors * FORMAT_ID_FIELD_LEN <= self.data_buf.len() {
                    let n = (self.args[ARG_FORMAT_SIZE_CODE] & SIZE_CODE_MASK) as usize;
                    let sector_size = if n > 0 {
                        SECTOR_SIZE_BASE << n
                    } else {
                        SECTOR_SIZE_BASE
                    };
                    let content = vec![self.args[ARG_FORMAT_FILLER]; sector_size];
                    let mut ids: Vec<(u8, u8, u8, u8)> = Vec::with_capacity(n_sectors);
                    while src_idx + FORMAT_ID_FIELD_LEN <= self.data_buf.len()
                        && ids.len() < n_sectors
                    {
                        let record = &self.data_buf[src_idx..src_idx + FORMAT_ID_FIELD_LEN];
                        let cyl = record[FORMAT_ID_CYLINDER_OFFSET];
                        let head = record[FORMAT_ID_HEAD_OFFSET];
                        let rec = record[FORMAT_ID_RECORD_OFFSET];
                        let size_code = record[FORMAT_ID_SIZE_CODE_OFFSET];
                        src_idx += FORMAT_ID_FIELD_LEN;
                        ids.push((cyl, head, rec, size_code));
                    }
                    done = self.format_track_on_drive(idx, self.arg_head(), &ids, &content);
                }
            }
        }

        if let Some(idx) = self.executing_drive {
            if self.data_buf.len() >= FORMAT_ID_FIELD_LEN {
                self.sector_id_cyl = self.data_buf[0];
                self.sector_id_head = self.data_buf[0];
                self.sector_id_rec = self.data_buf[0];
                self.sector_id_size_code = self.data_buf[0];
            }
            if done {
                src_idx -= FORMAT_ID_FIELD_LEN;
                if src_idx + FORMAT_ID_FIELD_LEN <= self.data_buf.len() {
                    let record = &self.data_buf[src_idx..src_idx + FORMAT_ID_FIELD_LEN];
                    self.sector_id_cyl = record[FORMAT_ID_CYLINDER_OFFSET];
                    self.sector_id_head = record[FORMAT_ID_HEAD_OFFSET];
                    self.sector_id_rec = record[FORMAT_ID_RECORD_OFFSET].wrapping_add(1);
                    self.sector_id_size_code = record[FORMAT_ID_SIZE_CODE_OFFSET];
                }
            } else {
                self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                if self.drives[idx].is_read_only() {
                    self.status_reg1 |= ST1_NOT_WRITABLE;
                } else {
                    self.status_reg1 |= ST1_DATA_ERROR;
                    self.status_reg2 |= ST2_DATA_ERROR_IN_DATA_FIELD;
                }
            }
            self.stop_execution();
        }
    }

    fn format_track_on_drive(
        &mut self,
        drive_idx: usize,
        head: usize,
        ids: &[(u8, u8, u8, u8)],
        content: &[u8],
    ) -> bool {
        if ids.is_empty() {
            return false;
        }
        for &(_cyl, _head, rec, _size_code) in ids {
            if !self.drives[drive_idx].format_sector(head, rec, content) {
                return false;
            }
        }
        true
    }

    fn exec_io_read_sector_by_id(&mut self) {
        let mut cyl_readable = false;
        let mut sector: Option<Sector> = None;
        let cm_abort = false;

        if let Some(idx) = self.executing_drive {
            let head = self.arg_head();
            let track_readable = self.drives[idx]
                .disk()
                .map(|disk| {
                    disk.is_hd() == self.hd_mode
                        && disk.sectors_of_track(self.drives[idx].cylinder() as usize, head) > 0
                })
                .unwrap_or(false);
            if track_readable {
                cyl_readable = true;
                sector = self.drives[idx].read_sector_by_id(
                    head,
                    0,
                    self.sector_id_cyl,
                    self.sector_id_head,
                    self.sector_id_rec,
                    self.sector_id_size_code,
                );
            }
        }

        if self.executing_drive.is_some() {
            match sector {
                Some(sector) => {
                    if sector.check_error() {
                        self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                        self.status_reg1 |= ST1_DATA_ERROR;
                        self.status_reg2 |= ST2_DATA_ERROR_IN_DATA_FIELD;
                    }
                    self.cur_sector_reader = Some(sector.reader());
                    self.cur_sector = Some(sector);
                    self.remain_bytes = self.data_len as i32;
                    self.start_io_req_timer();
                }
                None => {
                    self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                    if !cm_abort {
                        if cyl_readable {
                            if self.sector_id_cyl == self.args[ARG_CYLINDER]
                                && self.sector_id_head == self.args[ARG_HEAD]
                                && self.sector_id_rec == self.args[ARG_RECORD]
                            {
                                self.status_reg1 |= ST1_NO_DATA;
                            } else {
                                self.status_reg1 |= ST1_END_OF_CYLINDER;
                            }
                        } else {
                            self.status_reg1 |= ST1_MISSING_ADDRESS_MARK;
                        }
                    }
                    self.stop_execution();
                }
            }
        }
    }

    fn exec_io_read_sector_by_index(&mut self) {
        let mut cyl_readable = false;
        let mut sector: Option<Sector> = None;

        if let Some(idx) = self.executing_drive {
            let head = self.arg_head();
            let track_readable = self.drives[idx]
                .disk()
                .map(|disk| {
                    disk.is_hd() == self.hd_mode
                        && disk.sectors_of_track(self.drives[idx].cylinder() as usize, head) > 0
                })
                .unwrap_or(false);
            if track_readable {
                cyl_readable = true;
                sector =
                    self.drives[idx].read_sector_by_index(head, self.sector_id_rec as usize - 1);
            }
        }

        if self.executing_drive.is_some() {
            match sector {
                Some(sector) => {
                    if self.cur_cmd == Command::ReadTrack {
                        self.cur_sector_reader = Some(sector.reader());
                        self.cur_sector = Some(sector);
                        self.remain_bytes = self.data_len as i32;
                        self.set_byte_readable();
                    } else {
                        self.sector_id_cyl = sector.cylinder();
                        self.sector_id_head = sector.head();
                        self.sector_id_rec = sector.sector_num();
                        self.sector_id_size_code = sector.size_code();
                        self.cur_sector_reader = Some(sector.reader());
                        self.cur_sector = Some(sector);
                        self.remain_bytes = self.data_len as i32;
                        self.stop_execution();
                    }
                }
                None => {
                    self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                    if cyl_readable {
                        if self.sector_id_rec == 1 {
                            self.status_reg1 |= ST1_NO_DATA;
                        } else {
                            self.status_reg1 |= ST1_END_OF_CYLINDER;
                        }
                    } else {
                        self.status_reg1 |= ST1_MISSING_ADDRESS_MARK;
                    }
                    self.stop_execution();
                }
            }
        }
    }

    fn exec_io_read_sector_for_write(&mut self) {
        let mut cyl_readable = false;
        let mut sector: Option<Sector> = None;

        if let Some(idx) = self.executing_drive {
            let head = self.arg_head();
            let track_readable = self.drives[idx]
                .disk()
                .map(|disk| {
                    disk.is_hd() == self.hd_mode
                        && disk.sectors_of_track(self.drives[idx].cylinder() as usize, head) > 0
                })
                .unwrap_or(false);
            if track_readable {
                cyl_readable = true;
                sector = self.drives[idx].read_sector_by_id(
                    head,
                    0,
                    self.sector_id_cyl,
                    self.sector_id_head,
                    self.sector_id_rec,
                    self.sector_id_size_code,
                );
            }
        }

        if self.executing_drive.is_some() {
            match sector {
                Some(sector) => {
                    self.cur_sector = Some(sector);
                    self.data_pos = 0;
                    self.set_byte_writable(true);
                }
                None => {
                    self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                    if cyl_readable {
                        self.status_reg1 |= ST1_NO_DATA;
                    } else {
                        self.status_reg1 |= ST1_MISSING_ADDRESS_MARK;
                    }
                    self.stop_execution();
                }
            }
        }
    }

    fn exec_io_write_sector(&mut self) {
        let Some(idx) = self.executing_drive else {
            return;
        };
        let Some(disk_is_hd) = self.drives[idx].disk().map(FloppyDisk::is_hd) else {
            return;
        };
        let head = self.arg_head();
        let disk_ok = disk_is_hd == self.hd_mode;
        let sector = self.cur_sector.take();

        if disk_ok
            && let Some(sector) = &sector
            && !self.data_buf.is_empty()
            && self.data_pos >= 0
        {
            while (self.data_pos as usize) < self.data_len
                && (self.data_pos as usize) < self.data_buf.len()
            {
                self.data_buf[self.data_pos as usize] = 0;
                self.data_pos += 1;
            }
            let data_len = self.data_len;
            let buf = std::mem::take(&mut self.data_buf);
            let done = self.drives[idx].write_sector(head, sector, &buf, data_len);
            self.data_buf = buf;
            self.data_pos = DATA_POS_IDLE;

            if self.executing_drive.is_some() {
                if done {
                    self.inc_sector_num();
                    if self.tc_fired {
                        self.stop_execution();
                    } else {
                        self.start_io_task(IoTask::ReadSectorForWrite, self.tstates_per_rotation);
                    }
                } else {
                    self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                    if self.drives[idx].is_read_only() {
                        self.status_reg1 |= ST1_NOT_WRITABLE;
                    } else {
                        self.status_reg1 |= ST1_DATA_ERROR;
                        self.status_reg2 |= ST2_DATA_ERROR_IN_DATA_FIELD;
                    }
                    self.stop_execution();
                }
            }
        } else if self.tc_fired {
            self.cur_sector = sector;
            self.stop_execution();
        } else {
            self.cur_sector = sector;
        }
    }

    fn exec_seek_step(&mut self) {
        let mut seek_mode = false;
        for i in 0..NUM_DRIVES {
            let mut drive_seek_mode = false;
            if self.remain_seek_steps[i] > 0 {
                self.remain_seek_steps[i] -= 1;
                if self.drives[i].seek_step() {
                    self.seek_status[i] |= ST0_SEEK_END as i16;
                    self.interrupt_req = true;
                } else if self.remain_seek_steps[i] > 0 {
                    drive_seek_mode = true;
                } else {
                    self.seek_status[i] |= ST0_ABNORMAL_TERMINATION as i16;
                    self.seek_status[i] |= ST0_SEEK_END as i16;
                    self.seek_status[i] |= ST0_EQUIPMENT_CHECK as i16;
                    self.interrupt_req = true;
                }
            }
            if drive_seek_mode {
                seek_mode = true;
            } else {
                self.remain_seek_steps[i] = 0;
            }
        }
        self.seek_mode = seek_mode;
    }

    fn exec_sense_drive_status(&mut self) {
        self.status_reg3 = self.args[ARG_HEAD_DRIVE] & HEAD_DRIVE_MASK;
        let idx = self.arg_drive();
        self.status_reg3 |= ST3_TWO_SIDE;
        if self.drives[idx].cylinder() == 0 {
            self.status_reg3 |= ST3_TRACK_0;
        }
        if self.drives[idx].is_ready() {
            self.status_reg3 |= ST3_READY;
        }
        if self.drives[idx].is_read_only() {
            self.status_reg3 |= ST3_WRITE_PROTECTED;
        }
        self.results[0] = self.status_reg3;
        self.result_idx = 0;
        self.set_result_mode();
    }

    fn exec_sense_interrupt_status(&mut self) {
        self.results[0] = 0;
        self.results[1] = ST0_INVALID_COMMAND_ISSUE;

        let mut drive_mask: u8 = 0x01;
        for i in 0..NUM_DRIVES {
            let v = self.seek_status[i];
            if v >= 0 {
                if (v as u8 & ST0_SEEK_FLAGS_MASK) != 0 {
                    self.results[1] = v as u8 | i as u8;
                    self.results[0] = self.drives[i].cylinder() as u8;
                    self.status_reg_main &= !drive_mask;
                    self.seek_status[i] = SEEK_STATUS_IDLE;
                    break;
                }
                self.results[1] = 0;
            }
            drive_mask <<= 1;
        }
        self.result_idx = 1;
        self.set_result_mode();
    }

    fn sector_index_by_head_pos(&self, drive_idx: usize) -> i32 {
        let mut idx: i32 = -1;
        if let Some(disk) = self.drives[drive_idx].disk() {
            let head = self.arg_head();
            let cyl = self.drives[drive_idx].cylinder() as usize;
            let spc = disk.sectors_of_track(cyl, head);
            let tpr = self.tstates_per_rotation;
            if spc > 0 && tpr > 0 && head < disk.sides() && cyl < disk.cylinders() {
                idx = ((self.tstate_rotation_counter as f32 / tpr as f32) * spc as f32).round()
                    as i32;
                if idx >= spc as i32 {
                    idx = spc as i32 - 1;
                }
            }
            if idx < 0 {
                idx = 0;
            }
        }
        idx
    }

    fn read_from_disk(&mut self) -> i16 {
        let mut value: i16 = -1;
        if self.executing_drive.is_none() {
            return value;
        }
        if !matches!(
            self.cur_cmd,
            Command::ReadData | Command::ReadDeletedData | Command::ReadTrack
        ) {
            return value;
        }
        self.tstates_till_overrun = 0;

        if self.cur_sector.is_none() || self.cur_sector_reader.is_none() {
            return value;
        }
        if self.remain_bytes > 0 {
            let mut reader = self.cur_sector_reader.unwrap();
            if let Some(sector) = &self.cur_sector {
                value = reader.read(sector);
            }
            self.cur_sector_reader = Some(reader);
            self.remain_bytes -= 1;
        }

        if value < 0
            || (self.status_reg0 & ST0_ERROR_MASK) != 0
            || ((self.status_reg2 & ST2_CONTROL_MARK) != 0
                && (self.args[ARG_COMMAND] & ARG0_SK_MASK) == 0)
        {
            self.stop_execution();
        } else {
            self.status_reg_main &= !STM_REQUEST_FOR_MASTER;
            let byte_available = self
                .cur_sector_reader
                .map(|reader| reader.byte_available())
                .unwrap_or(false);
            if self.remain_bytes > 0 && byte_available {
                self.start_io_req_timer();
            } else {
                self.cur_sector = None;
                self.cur_sector_reader = None;
                self.inc_sector_num();
                let task = if self.cur_cmd == Command::ReadTrack {
                    IoTask::ReadSectorByIndex
                } else {
                    IoTask::ReadSectorById
                };
                self.start_io_task(
                    task,
                    self.tstates_per_rotation / NEXT_SECTOR_ROTATION_DIVISOR,
                );
            }
        }
        value
    }

    fn seek(&mut self, drive_num: usize, head: u8, cyl: u16) {
        self.status_reg_main &= !STM_BUSY;
        self.status_reg0 = 0;
        if drive_num < self.seek_status.len() {
            self.seek_status[drive_num] =
                (((head << HEAD_SHIFT) & HEAD_MASK) | drive_num as u8) as i16;
            if self.drives[drive_num].cylinder() == cyl {
                self.seek_status[drive_num] |= ST0_SEEK_END as i16;
                self.interrupt_req = true;
            } else {
                self.status_reg_main |= 1u8 << drive_num;
                self.drives[drive_num].set_seek_mode(head, cyl);
                self.remain_seek_steps[drive_num] = TRACKS_PER_DISK;
                if !self.seek_mode {
                    self.tstate_step_counter = 0;
                    self.seek_mode = true;
                }
            }
        } else {
            self.status_reg0 |= ST0_ABORT_BECAUSE_READY_CHANGED;
            self.status_reg0 |= ST0_SEEK_END;
            self.status_reg0 |= ST0_NOT_READY;
            self.status_reg0 |= (head << HEAD_SHIFT) & HEAD_MASK;
            self.status_reg0 |= drive_num as u8 & DRIVE_MASK;
            self.interrupt_req = true;
        }
    }

    fn set_byte_readable(&mut self) {
        if self.dma_mode {
            self.dma_req = true;
        } else {
            self.status_reg0 &= !HEAD_DRIVE_MASK;
            self.status_reg0 |= self.args[ARG_HEAD_DRIVE] & HEAD_DRIVE_MASK;
            self.interrupt_req = true;
        }
        self.status_reg_main |= STM_DATA_INPUT;
        self.status_reg_main |= STM_REQUEST_FOR_MASTER;
        self.start_overrun_timer();
    }

    fn set_byte_writable(&mut self, enable_overrun: bool) {
        if self.dma_mode {
            self.dma_req = true;
        } else {
            self.status_reg0 &= !HEAD_DRIVE_MASK;
            self.status_reg0 |= self.args[ARG_HEAD_DRIVE] & HEAD_DRIVE_MASK;
            self.interrupt_req = true;
        }
        self.status_reg_main &= !STM_DATA_INPUT;
        self.status_reg_main |= STM_REQUEST_FOR_MASTER;
        if enable_overrun {
            self.start_overrun_timer();
        }
    }

    fn set_data_buf(&mut self, data_len: usize) {
        self.data_len = data_len;
        if self.data_buf.len() < data_len {
            self.data_buf = vec![0; data_len];
        }
    }

    fn set_execution_mode(&mut self) {
        self.status_reg_main &= !STM_REQUEST_FOR_MASTER;
        self.status_reg_main &= !STM_DATA_INPUT;
        self.status_reg_main |= STM_BUSY;
        if !self.dma_mode {
            self.status_reg_main |= STM_NON_DMA_MODE;
        }
    }

    fn set_idle(&mut self) {
        self.status_reg_main &= STM_DRIVE_MASK;
        self.status_reg_main |= STM_REQUEST_FOR_MASTER;
        self.arg_idx = 0;
        self.result_idx = RESULT_IDLE;
        self.eot_reached = false;
        self.tc_enabled = false;
        self.tc_fired = false;
        self.executing_drive = None;
        self.cur_cmd = Command::Invalid;
    }

    fn set_result_mode(&mut self) {
        self.dma_req = false;
        self.tstates_till_io_req = 0;
        self.tstates_till_overrun = 0;
        self.status_reg_main &= STM_DRIVE_MASK;
        self.status_reg_main |= STM_BUSY;
        self.status_reg_main |= STM_DATA_INPUT;
        self.status_reg_main |= STM_REQUEST_FOR_MASTER;
    }

    fn start_format_track(&mut self) {
        self.set_execution_mode();
        self.clear_regs012();
        self.clear_sector_id();
        let mut done = false;
        let idx = self.arg_drive();
        if self.drives[idx].is_ready() {
            if self.drives[idx].is_read_only() {
                self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                self.status_reg1 |= ST1_NOT_WRITABLE;
            } else {
                self.set_data_buf(
                    self.args[ARG_FORMAT_SECTOR_COUNT] as usize * FORMAT_ID_FIELD_LEN,
                );
                self.data_pos = DATA_POS_IDLE;
                self.format_status = FormatStatus::WaitForHole;
                self.executing_drive = Some(idx);
                done = true;
            }
        }
        if !done {
            if (self.status_reg0 & ST0_ERROR_MASK) == 0 {
                self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                self.status_reg0 |= ST0_EQUIPMENT_CHECK;
                self.status_reg0 |= ST0_NOT_READY;
            }
            self.status_reg0 |= self.args[ARG_HEAD_DRIVE] & HEAD_DRIVE_MASK;
            self.stop_execution();
        }
    }

    fn start_io_req_timer(&mut self) {
        self.tstates_till_io_req =
            (self.tstates_per_milli as i32 / IO_REQ_MILLI_DIVISOR).min(IO_REQ_DELAY_CAP);
    }

    fn start_io_task(&mut self, task: IoTask, delay_tstates: i32) {
        self.io_task_cmd = task;
        self.tstates_till_io_start = delay_tstates;
    }

    fn start_overrun_timer(&mut self) {
        self.tstates_till_overrun = self.tstates_per_rotation;
    }

    fn start_read_data_or_scan(&mut self) {
        self.set_execution_mode();
        self.clear_regs012();
        self.sector_id_cyl = self.args[ARG_CYLINDER];
        self.sector_id_head = self.args[ARG_HEAD];
        self.sector_id_rec = self.args[ARG_RECORD];
        self.sector_id_size_code = self.args[ARG_SIZE_CODE];
        self.cur_sector_reader = None;
        self.tc_enabled = true;
        let mut done = false;
        let idx = self.arg_drive();
        if self.drives[idx].is_ready() {
            if matches!(
                self.cur_cmd,
                Command::ScanEqual | Command::ScanLowOrEqual | Command::ScanHighOrEqual
            ) {
                self.args[ARG_SIZE_CODE] &= SIZE_CODE_MASK;
                self.data_len = if self.args[ARG_SIZE_CODE] > 0 {
                    SECTOR_SIZE_BASE << self.args[ARG_SIZE_CODE]
                } else {
                    SECTOR_SIZE_BASE
                };
                self.status_reg2 |= ST2_SCAN_EQUAL_HIT;
            } else {
                self.data_len = self.arg_data_len();
            }
            self.executing_drive = Some(idx);
            self.start_io_task(IoTask::ReadSectorById, 0);
            done = true;
        }
        if !done {
            self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
            self.status_reg0 |= ST0_EQUIPMENT_CHECK;
            self.status_reg0 |= ST0_NOT_READY;
            self.status_reg0 |= self.args[ARG_HEAD_DRIVE] & HEAD_DRIVE_MASK;
            self.stop_execution();
        }
    }

    fn start_read_id(&mut self) {
        self.set_execution_mode();
        self.clear_regs012();
        self.clear_sector_id();
        let mut done = false;
        let idx = self.arg_drive();
        if self.drives[idx].is_ready() {
            let sector_idx = self.sector_index_by_head_pos(idx);
            if sector_idx >= 0 {
                self.executing_drive = Some(idx);
                self.sector_id_rec = (sector_idx + 1) as u8;
                self.start_io_task(IoTask::ReadSectorByIndex, 0);
                done = true;
            }
        }
        if !done {
            self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
            self.status_reg0 |= ST0_EQUIPMENT_CHECK;
            self.status_reg0 |= ST0_NOT_READY;
            self.status_reg0 |= self.args[ARG_HEAD_DRIVE] & HEAD_DRIVE_MASK;
            self.stop_execution();
        }
    }

    fn start_read_track(&mut self) {
        self.set_execution_mode();
        self.clear_regs012();
        self.sector_id_cyl = self.args[ARG_CYLINDER];
        self.sector_id_head = self.args[ARG_HEAD];
        self.sector_id_rec = 1;
        self.sector_id_size_code = self.args[ARG_SIZE_CODE];
        self.cur_sector_reader = None;
        self.tc_enabled = true;
        let mut done = false;
        let idx = self.arg_drive();
        if self.drives[idx].is_ready() {
            self.data_len = self.arg_data_len();
            self.executing_drive = Some(idx);
            let delay = (self.tstates_per_rotation - self.tstate_rotation_counter).max(0);
            self.start_io_task(IoTask::ReadSectorByIndex, delay);
            done = true;
        }
        if !done {
            self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
            self.status_reg0 |= ST0_EQUIPMENT_CHECK;
            self.status_reg0 |= ST0_NOT_READY;
            self.status_reg0 |= self.args[ARG_HEAD_DRIVE] & HEAD_DRIVE_MASK;
            self.stop_execution();
        }
    }

    fn start_write_data(&mut self) {
        self.set_execution_mode();
        self.clear_regs012();
        self.sector_id_cyl = self.args[ARG_CYLINDER];
        self.sector_id_head = self.args[ARG_HEAD];
        self.sector_id_rec = self.args[ARG_RECORD];
        self.sector_id_size_code = self.args[ARG_SIZE_CODE];
        self.cur_sector = None;
        self.tc_enabled = true;
        let mut done = false;
        let idx = self.arg_drive();
        if self.drives[idx].is_ready() {
            if self.drives[idx].is_read_only() {
                self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                self.status_reg1 |= ST1_NOT_WRITABLE;
            } else {
                if !self.dma_mode {
                    self.status_reg_main |= STM_NON_DMA_MODE;
                }
                self.set_data_buf(self.arg_data_len());
                self.data_pos = DATA_POS_IDLE;
                self.executing_drive = Some(idx);
                self.start_io_task(IoTask::ReadSectorForWrite, 0);
                done = true;
            }
        }
        if !done {
            if (self.status_reg0 & ST0_ERROR_MASK) == 0 {
                self.status_reg0 |= ST0_ABNORMAL_TERMINATION;
                self.status_reg0 |= ST0_EQUIPMENT_CHECK;
                self.status_reg0 |= ST0_NOT_READY;
            }
            self.status_reg0 |= self.args[ARG_HEAD_DRIVE] & HEAD_DRIVE_MASK;
            self.stop_execution();
        }
    }

    fn stop_execution(&mut self) {
        if !self.eot_reached {
            self.tc_enabled = false;
        }
        self.io_task_cmd = IoTask::Idle;
        self.tc_fired = false;
        self.executing_drive = None;
        self.format_status = FormatStatus::Idle;
        self.results[0] = self.sector_id_size_code;
        self.results[1] = self.sector_id_rec;
        self.results[2] = self.sector_id_head;
        self.results[3] = self.sector_id_cyl;
        self.results[4] = self.status_reg2;
        self.results[5] = self.status_reg1;
        self.results[6] = self.status_reg0;
        self.result_idx = 6;
        self.status_reg0 &= !HEAD_DRIVE_MASK;
        self.status_reg0 |= self.args[ARG_HEAD_DRIVE] & HEAD_DRIVE_MASK;
        self.interrupt_req = true;
        self.set_result_mode();
    }

    fn write_cmd(&mut self, value: u8) {
        self.status_reg_main |= STM_BUSY;
        if self.arg_idx == ARG_PHASE_COMMAND {
            self.executing_drive = None;
            self.result_idx = RESULT_IDLE;
            self.args[self.arg_idx] = value;
            self.arg_idx += 1;
            self.cur_cmd = COMMAND_TABLE[(value & COMMAND_MASK) as usize];
            if self.cur_cmd == Command::Invalid {
                self.interrupt_req = false;
                self.status_reg0 = ST0_INVALID_COMMAND_ISSUE;
                self.results[0] = self.status_reg0;
                self.result_idx = 0;
                self.set_result_mode();
            } else if self.cur_cmd == Command::SenseInterruptStatus {
                self.exec_sense_interrupt_status();
            }
        } else if self.executing_drive.is_none() && self.result_idx < 0 {
            if self.arg_idx < self.args.len() {
                self.args[self.arg_idx] = value;
                self.arg_idx += 1;
            }
            match self.cur_cmd {
                Command::FormatTrack => {
                    if self.arg_idx == ARG_PHASE_FORMAT {
                        self.start_format_track();
                    }
                }
                Command::ReadData
                | Command::ReadDeletedData
                | Command::ScanEqual
                | Command::ScanLowOrEqual
                | Command::ScanHighOrEqual => {
                    if self.arg_idx == ARG_PHASE_READ_WRITE_SCAN {
                        self.start_read_data_or_scan();
                    }
                }
                Command::ReadId => {
                    if self.arg_idx == ARG_PHASE_READ_ID {
                        self.start_read_id();
                    }
                }
                Command::ReadTrack => {
                    if self.arg_idx == ARG_PHASE_READ_WRITE_SCAN {
                        self.start_read_track();
                    }
                }
                Command::Recalibrate => {
                    if self.arg_idx == ARG_PHASE_RECALIBRATE {
                        let drive = (self.args[ARG_HEAD_DRIVE] & DRIVE_MASK) as usize;
                        self.seek(drive, 0, 0);
                        self.set_idle();
                    }
                }
                Command::Seek => {
                    if self.arg_idx == ARG_PHASE_SEEK {
                        let drive = (self.args[ARG_HEAD_DRIVE] & DRIVE_MASK) as usize;
                        let head = (self.args[ARG_HEAD_DRIVE] >> HEAD_SHIFT) & HEAD_SELECT_MASK;
                        let cyl = self.args[ARG_CYLINDER] as u16;
                        self.seek(drive, head, cyl);
                        self.set_idle();
                    }
                }
                Command::SenseDriveStatus => {
                    if self.arg_idx == ARG_PHASE_SENSE_DRIVE {
                        self.exec_sense_drive_status();
                    }
                }
                Command::Specify => {
                    if self.arg_idx == ARG_PHASE_SPECIFY {
                        self.step_rate_millis = SPECIFY_STEP_RATE_BASE
                            - ((self.args[ARG_HEAD_DRIVE] >> SPECIFY_STEP_RATE_SHIFT)
                                & SPECIFY_STEP_RATE_MASK) as u32;
                        self.dma_mode = (self.args[ARG_CYLINDER] & SPECIFY_NON_DMA_BIT) == 0;
                        self.calc_tstates_per_step();
                        self.set_idle();
                    }
                }
                Command::WriteData | Command::WriteDeletedData => {
                    if self.arg_idx == ARG_PHASE_READ_WRITE_SCAN {
                        self.start_write_data();
                    }
                }
                _ => self.set_idle(),
            }
        }
    }

    fn write_to_drive(&mut self, value: u8) {
        self.status_reg_main &= !STM_REQUEST_FOR_MASTER;
        if self.executing_drive.is_none() {
            return;
        }

        if matches!(
            self.cur_cmd,
            Command::ScanEqual | Command::ScanLowOrEqual | Command::ScanHighOrEqual
        ) {
            self.tstates_till_overrun = 0;
            if self.cur_sector.is_none() || self.cur_sector_reader.is_none() {
                return;
            }
            let mut b: i16 = -1;
            if self.remain_bytes > 0 {
                let mut reader = self.cur_sector_reader.unwrap();
                if let Some(sector) = &self.cur_sector {
                    b = reader.read(sector);
                }
                self.cur_sector_reader = Some(reader);
                self.remain_bytes -= 1;
            }
            if b < 0 || (self.status_reg0 & ST0_ERROR_MASK) != 0 {
                self.stop_execution();
                return;
            }
            if b != value as i16 {
                self.status_reg2 &= !ST2_SCAN_EQUAL_HIT;
            }
            match self.cur_cmd {
                Command::ScanLowOrEqual => {
                    if b > value as i16 {
                        self.status_reg2 |= ST2_SCAN_NOT_SATISFIED;
                    }
                }
                Command::ScanHighOrEqual => {
                    if b < value as i16 {
                        self.status_reg2 |= ST2_SCAN_NOT_SATISFIED;
                    }
                }
                _ => {
                    if b != value as i16 {
                        self.status_reg2 |= ST2_SCAN_NOT_SATISFIED;
                    }
                }
            }
            let byte_available = self
                .cur_sector_reader
                .map(|reader| reader.byte_available())
                .unwrap_or(false);
            if self.remain_bytes > 0 && byte_available {
                self.start_io_req_timer();
            } else if (self.status_reg0 & ST0_ERROR_MASK) != 0 {
                self.stop_execution();
            } else {
                self.cur_sector = None;
                self.cur_sector_reader = None;
                self.add_sector_num(self.args[ARG_SCAN_STEP]);
                self.start_io_task(
                    IoTask::ReadSectorById,
                    self.tstates_per_rotation / NEXT_SECTOR_ROTATION_DIVISOR,
                );
            }
        } else if !self.data_buf.is_empty()
            && self.data_pos >= 0
            && (self.data_pos as usize) < self.data_buf.len()
            && (self.data_pos as usize) < self.data_len
        {
            if self.cur_cmd == Command::FormatTrack {
                self.data_buf[self.data_pos as usize] = value;
                self.data_pos += 1;
                self.start_io_req_timer();
            } else if matches!(self.cur_cmd, Command::WriteData | Command::WriteDeletedData) {
                self.tstates_till_overrun = 0;
                self.data_buf[self.data_pos as usize] = value;
                self.data_pos += 1;
                if (self.data_pos as usize) < self.data_buf.len()
                    && (self.data_pos as usize) < self.data_len
                {
                    self.start_io_req_timer();
                } else {
                    self.start_io_task(IoTask::WriteSector, 0);
                }
            }
        }
    }
}
