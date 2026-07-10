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

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;

const HEAD_MASK: usize = 0x01;
const SIZE_CODE_BASE: usize = 0x80;
const SIZE_CODE_MAX: u8 = 6;
const HD_SECTOR_SIZE: usize = 512;
const HD_SECTORS_PER_TRACK: usize = 15;

const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];
const COPYQM_MAGIC: [u8; 3] = [b'C', b'Q', 0x14];
const IMAGEDISK_MAGIC: &[u8] = b"IMD ";
const TELEDISK_MAGIC_PLAIN: [u8; 2] = *b"TD";
const TELEDISK_MAGIC_PACKED: [u8; 2] = *b"td";
const CPC_HEADER_STD: &[u8] = b"MV - CPCEMU";
const CPC_HEADER_EXT: &[u8] = b"EXTENDED CPC DSK";
const CPC_TRACK_HEADER: &[u8] = b"Track-Info\r\n";
const TWO_SIDES: usize = 2;
const MAX_IMAGE_SIZE: usize = 4 * 1024 * 1024;
const IMD_END_OF_COMMENT: u8 = 0x1A;
const IMD_HEAD_CYL_MAP: u8 = 0x80;
const IMD_HEAD_HEAD_MAP: u8 = 0x40;
const IMD_HEAD_NUMBER_MASK: u8 = 0x01;
const TELEDISK_SUPPORTED_VERSION: u8 = 0x15;
const TELEDISK_HEADER_LEN: usize = 12;
const TELEDISK_VERSION_OFFSET: usize = 4;
const TELEDISK_STEPPING_OFFSET: usize = 7;
const TELEDISK_HEAD_NUMBER_MASK: u8 = 0x01;
const TELEDISK_REMARK_FLAG: u8 = 0x80;
const TELEDISK_SECTOR_CRC_ERROR: u8 = 0x02;
const TELEDISK_SECTOR_DELETED: u8 = 0x04;
const TELEDISK_SECTOR_BOGUS_HEADER: u8 = 0x40;
const TELEDISK_SECTOR_NO_DATA_MASK: u8 = 0x30;
const TELEDISK_SECTOR_PHANTOM: u8 = 0xFF;
const CPC_FILE_HEADER_LEN: usize = 0x100;
const CPC_TRACK_HEADER_LEN: usize = 0x100;
const CPC_CYL_COUNT_OFFSET: usize = 0x30;
const CPC_SIDE_COUNT_OFFSET: usize = 0x31;
const CPC_STD_TRACK_SIZE_OFFSET: usize = 0x32;
const CPC_EXT_TRACK_SIZE_TABLE: usize = 0x34;
const CPC_TRACK_CYL_OFFSET: usize = 0x10;
const CPC_TRACK_SIDE_OFFSET: usize = 0x11;
const CPC_TRACK_SIZE_CODE_OFFSET: usize = 0x14;
const CPC_TRACK_SECTOR_COUNT_OFFSET: usize = 0x15;
const CPC_TRACK_SECTOR_LIST_OFFSET: usize = 0x18;
const CPC_SECTOR_INFO_LEN: usize = 8;
const CPC_SECTOR_HEAD_OFFSET: usize = 1;
const CPC_SECTOR_ID_OFFSET: usize = 2;
const CPC_SECTOR_SIZE_CODE_OFFSET: usize = 3;
const CPC_SECTOR_ACTUAL_LEN_OFFSET: usize = 6;
const CPC_BIG_SECTOR_SIZE: usize = 0x1800;
const COPYQM_HEADER_LEN: usize = 133;
const COPYQM_SECTOR_SIZE_OFFSET: usize = 3;
const COPYQM_SECTORS_PER_TRACK_OFFSET: usize = 0x10;
const COPYQM_SIDES_OFFSET: usize = 0x12;
const COPYQM_USED_CYLS_OFFSET: usize = 0x5A;
const COPYQM_CYLS_OFFSET: usize = 0x5B;
const COPYQM_COMMENT_LEN_OFFSET: usize = 0x6F;
const COPYQM_SECTOR_OFFSET_OFFSET: usize = 0x71;
const COPYQM_RUN_FLAG: usize = 0x8000;
const COPYQM_RUN_MODULO: usize = 0x1_0000;
const COPYQM_WORD_MASK: usize = 0xFFFF;

const CPM_RECORD_SIZE: usize = 128;
const CPM_RECORDS_PER_EXTENT: usize = 128;
const CPM_EXTENT_SIZE: usize = CPM_RECORDS_PER_EXTENT * CPM_RECORD_SIZE;
const CPM_DIR_ENTRY_LEN: usize = 32;
const CPM_FILL_BYTE: u8 = 0xE5;
const CPM_NAME_MASK: u8 = 0x7F;
const CPM_MAX_USER: u8 = 15;
const CPM_NAME_OFFSET: usize = 1;
const CPM_NAME_LEN: usize = 8;
const CPM_EXT_OFFSET: usize = 9;
const CPM_EXT_LEN: usize = 3;
const CPM_EXTENT_LOW_OFFSET: usize = 12;
const CPM_EXTENT_LOW_MASK: usize = 0x1F;
const CPM_EXTENT_HIGH_OFFSET: usize = 14;
const CPM_EXTENT_HIGH_SHIFT: u32 = 5;
const CPM_EXTENT_HIGH_MASK: usize = 0x07E0;
const CPM_EXTENT_HIGH_STORE_MASK: usize = 0x3F;
const CPM_RECORD_COUNT_OFFSET: usize = 15;
const CPM_BLOCK_LIST_OFFSET: usize = 16;
const CPM_BLOCK_LIST_LEN: usize = 16;
const CPM_ATTR_BIT: u8 = 0x80;
const CPM_MANIFEST_NAME: &str = "manifest.json";
const CPM_USER_DIR_PREFIX: &str = "user";

const CPM_DS_FILENAME: &str = "!!!TIME&.DAT";
const CPM_DS_RECORD_LEN: usize = 16;
const CPM_DS_SIGNATURE: [u8; 8] = [b'!', b'!', b'!', b'T', b'I', b'M', b'E', 0x92];
const CPM_DS_CHECKSUM_SPAN: usize = 0x7F;
const CPM_DS_SIGNATURE_OFFSET: usize = 0x0F;
const CPM_DS_STAMP_LEN: usize = 5;
const CPM_DS_STAMP_COUNT: usize = 3;
const CPM_DS_YEAR_MIN: i64 = 1978;
const CPM_DS_YEAR_MAX: i64 = 2078;

const SECONDS_PER_DAY: i64 = 86_400;
const SECONDS_PER_HOUR: i64 = 3600;
const SECONDS_PER_MINUTE: i64 = 60;

const DIR_REFRESH_SECONDS: u64 = 5;
const DIR_WRITE_GRACE_MILLIS: u64 = 1000;

#[derive(Clone, Copy)]
pub struct DiskFormat {
    pub cylinders: usize,
    pub sides: usize,
    pub sectors_per_track: usize,
    pub sector_size: usize,
    pub interleave: usize,
}

impl DiskFormat {
    pub const LLC2_400K: DiskFormat = DiskFormat::new(80, 1, 5, 1024, 1);
    pub const PCM_624K: DiskFormat = DiskFormat::new(80, 2, 16, 256, 1);
    pub const MLDOS_702K_I3: DiskFormat = DiskFormat::new(80, 2, 9, 512, 3);
    pub const BASDOS_711K_I5: DiskFormat = DiskFormat::new(80, 2, 9, 512, 5);
    pub const A5105_720K: DiskFormat = DiskFormat::new(80, 2, 9, 512, 1);
    pub const CAOS_780K: DiskFormat = DiskFormat::new(80, 2, 5, 1024, 1);
    pub const SCPX_780K_I2: DiskFormat = DiskFormat::new(80, 2, 5, 1024, 2);
    pub const MICRODOS_780K_I3: DiskFormat = DiskFormat::new(80, 2, 5, 1024, 3);
    pub const CPA_800K: DiskFormat = DiskFormat::new(80, 2, 5, 1024, 4);
    pub const MSDOS_1200K: DiskFormat = DiskFormat::new(80, 2, 15, 512, 1);
    pub const MSDOS_1440K: DiskFormat = DiskFormat::new(80, 2, 18, 512, 1);
    pub const MLDOS_1738K_I3: DiskFormat = DiskFormat::new(80, 2, 11, 1024, 3);

    pub const NAMED: [(&'static str, DiskFormat); 12] = [
        ("llc2-400k", Self::LLC2_400K),
        ("pcm-624k", Self::PCM_624K),
        ("mldos-702k", Self::MLDOS_702K_I3),
        ("basdos-711k", Self::BASDOS_711K_I5),
        ("a5105-720k", Self::A5105_720K),
        ("caos-780k", Self::CAOS_780K),
        ("scpx-780k", Self::SCPX_780K_I2),
        ("microdos-780k", Self::MICRODOS_780K_I3),
        ("z9001-800k", Self::CPA_800K),
        ("msdos-1200k", Self::MSDOS_1200K),
        ("msdos-1440k", Self::MSDOS_1440K),
        ("mldos-1738k", Self::MLDOS_1738K_I3),
    ];

    pub fn by_name(name: &str) -> Option<DiskFormat> {
        Self::NAMED
            .iter()
            .find(|(candidate, _)| *candidate == name)
            .map(|(_, format)| *format)
    }

    pub const fn new(
        cylinders: usize,
        sides: usize,
        sectors_per_track: usize,
        sector_size: usize,
        interleave: usize,
    ) -> Self {
        Self {
            cylinders,
            sides,
            sectors_per_track,
            sector_size,
            interleave,
        }
    }

    pub fn image_size(&self) -> usize {
        self.cylinders * self.sides * self.sectors_per_track * self.sector_size
    }

    pub fn to_disk(&self, bytes: Vec<u8>, read_only: bool) -> FloppyDisk {
        let physical_order = interleave_order(self.sectors_per_track, self.interleave);
        let mut tracks = Vec::with_capacity(self.cylinders * self.sides);
        for cyl in 0..self.cylinders {
            for head in 0..self.sides {
                let mut sectors = Vec::with_capacity(self.sectors_per_track);
                for &logical in &physical_order {
                    let block = ((self.sides * cyl) + head) * self.sectors_per_track + logical;
                    let start = block * self.sector_size;
                    let mut data = vec![0u8; self.sector_size];
                    if start < bytes.len() {
                        let end = (start + self.sector_size).min(bytes.len());
                        data[..end - start].copy_from_slice(&bytes[start..end]);
                    }
                    sectors.push(StoredSector::new(
                        cyl as u8,
                        head as u8,
                        (logical + 1) as u8,
                        size_code_by_size(self.sector_size),
                        data,
                    ));
                }
                tracks.push(sectors);
            }
        }
        FloppyDisk::from_tracks(
            tracks,
            self.cylinders,
            self.sides,
            self.sectors_per_track,
            self.sector_size,
            read_only,
        )
    }
}

fn size_code_by_size(sector_size: usize) -> u8 {
    if sector_size <= SIZE_CODE_BASE {
        return 0;
    }
    let mut value = 0u8;
    let mut size = SIZE_CODE_BASE;
    while size < sector_size && value < SIZE_CODE_MAX {
        value += 1;
        size <<= 1;
    }
    value
}

fn size_by_size_code(size_code: u8) -> usize {
    SIZE_CODE_BASE << (size_code.min(SIZE_CODE_MAX) as usize)
}

fn interleave_order(sectors_per_track: usize, interleave: usize) -> Vec<usize> {
    if interleave <= 1 || interleave >= sectors_per_track || sectors_per_track <= 2 {
        return (0..sectors_per_track).collect();
    }
    let mut map = vec![usize::MAX; sectors_per_track];
    let mut src_idx = 0;
    let mut dst_idx = 0;
    while src_idx < sectors_per_track {
        while map[dst_idx] != usize::MAX {
            dst_idx = (dst_idx + 1) % sectors_per_track;
        }
        map[dst_idx] = src_idx;
        src_idx += 1;
        dst_idx = (dst_idx + interleave) % sectors_per_track;
    }
    map
}

#[derive(Clone)]
struct StoredSector {
    pub cylinder: u8,
    pub head: u8,
    pub sector_num: u8,
    pub size_code: u8,
    pub data: Vec<u8>,
    pub deleted: bool,
    pub error: bool,
    pub file_offset: Option<u64>,
}

impl StoredSector {
    pub fn new(cylinder: u8, head: u8, sector_num: u8, size_code: u8, data: Vec<u8>) -> Self {
        Self {
            cylinder,
            head,
            sector_num,
            size_code,
            data,
            deleted: false,
            error: false,
            file_offset: None,
        }
    }

    fn with_offset(mut self, offset: u64) -> Self {
        self.file_offset = Some(offset);
        self
    }
}

#[derive(Clone)]
pub struct Sector {
    index_on_cylinder: usize,
    cylinder: u8,
    head: u8,
    sector_num: u8,
    size_code: u8,
    data: Vec<u8>,
    deleted: bool,
    error: bool,
}

impl Sector {
    pub fn cylinder(&self) -> u8 {
        self.cylinder
    }

    pub fn head(&self) -> u8 {
        self.head
    }

    pub fn sector_num(&self) -> u8 {
        self.sector_num
    }

    pub fn size_code(&self) -> u8 {
        self.size_code
    }

    pub fn index_on_cylinder(&self) -> usize {
        self.index_on_cylinder
    }

    pub fn data_deleted(&self) -> bool {
        self.deleted
    }

    pub fn check_error(&self) -> bool {
        self.error
    }

    pub fn reader(&self) -> SectorReader {
        SectorReader {
            pos: 0,
            remaining: self.data.len(),
            deleted: self.deleted,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SectorReader {
    pos: usize,
    remaining: usize,
    deleted: bool,
}

impl SectorReader {
    pub fn byte_available(&self) -> bool {
        self.remaining > 0
    }

    pub fn data_deleted(&self) -> bool {
        self.deleted
    }

    pub fn read(&mut self, sector: &Sector) -> i16 {
        let mut value: i16 = -1;
        if self.remaining > 0 {
            if self.pos < sector.data.len() {
                value = sector.data[self.pos] as i16;
                self.pos += 1;
                self.remaining -= 1;
            } else {
                value = 0;
                self.remaining -= 1;
            }
        }
        value
    }
}

pub trait DiskBackend: Send {
    fn cylinders(&self) -> usize;
    fn sides(&self) -> usize;
    fn sectors_of_track(&self, phys_cyl: usize, phys_head: usize) -> usize;
    fn is_hd(&self) -> bool;
    fn is_read_only(&self) -> bool;

    fn sector_by_index(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector_idx: usize,
    ) -> Option<Sector>;

    fn sector_by_id(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        cyl: u8,
        head: u8,
        sector_num: u8,
        size_code: i16,
    ) -> Option<Sector>;

    fn write_sector(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector: &Sector,
        data: &[u8],
        data_len: usize,
    ) -> bool;

    fn format_sector(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector_num: u8,
        content: &[u8],
    ) -> bool;
}

pub struct FloppyDisk {
    tracks: Vec<Vec<StoredSector>>,
    cylinders: usize,
    sides: usize,
    primary_sectors_per_track: usize,
    primary_sector_size: usize,
    read_only: bool,
}

impl FloppyDisk {
    fn from_tracks(
        tracks: Vec<Vec<StoredSector>>,
        cylinders: usize,
        sides: usize,
        primary_sectors_per_track: usize,
        primary_sector_size: usize,
        read_only: bool,
    ) -> Self {
        Self {
            tracks,
            cylinders,
            sides,
            primary_sectors_per_track,
            primary_sector_size,
            read_only,
        }
    }

    fn track_index(&self, phys_cyl: usize, phys_head: usize) -> Option<usize> {
        let head = phys_head & HEAD_MASK;
        if phys_cyl >= self.cylinders || head >= self.sides {
            return None;
        }
        Some(phys_cyl * self.sides + head)
    }

    fn track(&self, phys_cyl: usize, phys_head: usize) -> Option<&[StoredSector]> {
        self.track_index(phys_cyl, phys_head)
            .map(|idx| self.tracks[idx].as_slice())
    }

    fn sector_file_offset(&self, phys_cyl: usize, phys_head: usize, sector_num: u8) -> Option<u64> {
        self.track(phys_cyl, phys_head)?
            .iter()
            .find(|stored| stored.sector_num == sector_num)
            .and_then(|stored| stored.file_offset)
    }

    fn build_sector(stored: &StoredSector, index_on_cylinder: usize) -> Sector {
        Sector {
            index_on_cylinder,
            cylinder: stored.cylinder,
            head: stored.head,
            sector_num: stored.sector_num,
            size_code: stored.size_code,
            data: stored.data.clone(),
            deleted: stored.deleted,
            error: stored.error,
        }
    }

    fn linear_bytes(&self) -> Vec<u8> {
        let spt = self.primary_sectors_per_track;
        let size = self.primary_sector_size;
        let total = self.cylinders * self.sides * spt * size;
        let mut out = vec![0u8; total];
        for (track_idx, track) in self.tracks.iter().enumerate() {
            let phys_cyl = track_idx / self.sides;
            let phys_head = track_idx % self.sides;
            for stored in track {
                if stored.sector_num == 0 {
                    continue;
                }
                let logical = stored.sector_num as usize - 1;
                if logical >= spt {
                    continue;
                }
                let block = (phys_cyl * self.sides + phys_head) * spt + logical;
                let offset = block * size;
                let len = stored.data.len().min(size);
                if offset + len <= out.len() {
                    out[offset..offset + len].copy_from_slice(&stored.data[..len]);
                }
            }
        }
        out
    }
}

impl DiskBackend for FloppyDisk {
    fn cylinders(&self) -> usize {
        self.cylinders
    }

    fn sides(&self) -> usize {
        self.sides
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }

    fn is_hd(&self) -> bool {
        self.primary_sectors_per_track >= HD_SECTORS_PER_TRACK
            && self.primary_sector_size <= HD_SECTOR_SIZE
    }

    fn sectors_of_track(&self, phys_cyl: usize, phys_head: usize) -> usize {
        self.track(phys_cyl, phys_head).map_or(0, <[_]>::len)
    }

    fn sector_by_index(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector_idx: usize,
    ) -> Option<Sector> {
        let track = self.track(phys_cyl, phys_head)?;
        track
            .get(sector_idx)
            .map(|stored| Self::build_sector(stored, sector_idx))
    }

    fn sector_by_id(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        cyl: u8,
        head: u8,
        sector_num: u8,
        size_code: i16,
    ) -> Option<Sector> {
        let track = self.track(phys_cyl, phys_head)?;
        track.iter().enumerate().find_map(|(idx, stored)| {
            if stored.cylinder == cyl
                && stored.head == head
                && stored.sector_num == sector_num
                && (size_code < 0 || stored.size_code as i16 == size_code)
            {
                Some(Self::build_sector(stored, idx))
            } else {
                None
            }
        })
    }

    fn write_sector(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector: &Sector,
        data: &[u8],
        data_len: usize,
    ) -> bool {
        if self.read_only {
            return false;
        }
        let Some(track_idx) = self.track_index(phys_cyl, phys_head) else {
            return false;
        };
        let Some(stored) = self.tracks[track_idx].get_mut(sector.index_on_cylinder) else {
            return false;
        };
        if stored.sector_num != sector.sector_num || data_len != stored.data.len() {
            return false;
        }
        stored.data.copy_from_slice(&data[..data_len]);
        stored.deleted = false;
        stored.error = false;
        true
    }

    fn format_sector(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector_num: u8,
        content: &[u8],
    ) -> bool {
        if self.read_only {
            return false;
        }
        let Some(track_idx) = self.track_index(phys_cyl, phys_head) else {
            return false;
        };
        let Some(stored) = self.tracks[track_idx]
            .iter_mut()
            .find(|stored| stored.sector_num == sector_num)
        else {
            return false;
        };
        if content.len() != stored.data.len() {
            return false;
        }
        stored.data.copy_from_slice(content);
        stored.deleted = false;
        stored.error = false;
        true
    }
}

struct RawGeometry {
    sectors_per_track: usize,
    sector_size: usize,
    sides: usize,
}

impl RawGeometry {
    fn offset(&self, phys_cyl: usize, phys_head: usize, sector_num: u8) -> Option<u64> {
        if sector_num == 0 {
            return None;
        }
        let logical = sector_num as usize - 1;
        if logical >= self.sectors_per_track || phys_head >= self.sides {
            return None;
        }
        let block = (phys_cyl * self.sides + phys_head) * self.sectors_per_track + logical;
        Some((block * self.sector_size) as u64)
    }
}

pub struct FileImageDisk {
    image: FloppyDisk,
    file: File,
    raw_geometry: Option<RawGeometry>,
}

impl FileImageDisk {
    pub fn open_raw(path: &Path, format: DiskFormat) -> std::io::Result<Self> {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        file.seek(SeekFrom::Start(0))?;
        let image = format.to_disk(bytes, false);
        Ok(Self {
            image,
            file,
            raw_geometry: Some(RawGeometry {
                sectors_per_track: format.sectors_per_track,
                sector_size: format.sector_size,
                sides: format.sides,
            }),
        })
    }

    pub fn open_container(path: &Path, image: FloppyDisk) -> std::io::Result<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        Ok(Self {
            image,
            file,
            raw_geometry: None,
        })
    }

    fn flush_sector(&mut self, phys_cyl: usize, phys_head: usize, sector_num: u8, data: &[u8]) {
        let offset = self
            .image
            .sector_file_offset(phys_cyl, phys_head, sector_num)
            .or_else(|| {
                self.raw_geometry
                    .as_ref()
                    .and_then(|geometry| geometry.offset(phys_cyl, phys_head, sector_num))
            });
        let Some(offset) = offset else {
            return;
        };
        if self.file.seek(SeekFrom::Start(offset)).is_ok() {
            let _ = self.file.write_all(data);
            let _ = self.file.flush();
        }
    }
}

impl DiskBackend for FileImageDisk {
    fn cylinders(&self) -> usize {
        self.image.cylinders()
    }

    fn sides(&self) -> usize {
        self.image.sides()
    }

    fn sectors_of_track(&self, phys_cyl: usize, phys_head: usize) -> usize {
        self.image.sectors_of_track(phys_cyl, phys_head)
    }

    fn is_hd(&self) -> bool {
        self.image.is_hd()
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn sector_by_index(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector_idx: usize,
    ) -> Option<Sector> {
        self.image.sector_by_index(phys_cyl, phys_head, sector_idx)
    }

    fn sector_by_id(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        cyl: u8,
        head: u8,
        sector_num: u8,
        size_code: i16,
    ) -> Option<Sector> {
        self.image
            .sector_by_id(phys_cyl, phys_head, cyl, head, sector_num, size_code)
    }

    fn write_sector(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector: &Sector,
        data: &[u8],
        data_len: usize,
    ) -> bool {
        if !self
            .image
            .write_sector(phys_cyl, phys_head, sector, data, data_len)
        {
            return false;
        }
        self.flush_sector(phys_cyl, phys_head, sector.sector_num(), &data[..data_len]);
        true
    }

    fn format_sector(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector_num: u8,
        content: &[u8],
    ) -> bool {
        if !self
            .image
            .format_sector(phys_cyl, phys_head, sector_num, content)
        {
            return false;
        }
        self.flush_sector(phys_cyl, phys_head, sector_num, content);
        true
    }
}

#[derive(Clone, Copy)]
struct CpmParams {
    block_size: usize,
    dir_blocks: usize,
    sys_bytes: usize,
    block_num_16bit: bool,
    datestamp: bool,
}

fn cpm_params(format: DiskFormat) -> Option<CpmParams> {
    let sys_bytes = |sys_tracks: usize| {
        sys_tracks * format.sides * format.sectors_per_track * format.sector_size
    };
    let size = format.image_size();
    let params = if size == DiskFormat::CPA_800K.image_size() {
        CpmParams {
            block_size: 2048,
            dir_blocks: 3,
            sys_bytes: sys_bytes(0),
            block_num_16bit: true,
            datestamp: false,
        }
    } else if size == DiskFormat::MSDOS_1200K.image_size()
        || size == DiskFormat::MSDOS_1440K.image_size()
    {
        CpmParams {
            block_size: 4096,
            dir_blocks: 2,
            sys_bytes: sys_bytes(0),
            block_num_16bit: true,
            datestamp: false,
        }
    } else if size == DiskFormat::MLDOS_1738K_I3.image_size() {
        CpmParams {
            block_size: 4096,
            dir_blocks: 2,
            sys_bytes: sys_bytes(1),
            block_num_16bit: true,
            datestamp: true,
        }
    } else {
        return None;
    };
    Some(params)
}

fn cpm_block_pointer_count(block_num_16bit: bool) -> usize {
    if block_num_16bit {
        CPM_BLOCK_LIST_LEN / 2
    } else {
        CPM_BLOCK_LIST_LEN
    }
}

fn cpm_blocks_per_extent(block_size: usize, block_num_16bit: bool) -> usize {
    (CPM_EXTENT_SIZE / block_size).min(cpm_block_pointer_count(block_num_16bit))
}

fn cpm_records_per_block(block_size: usize) -> usize {
    block_size / CPM_RECORD_SIZE
}

fn cpm_clean_name(raw: &[u8]) -> String {
    let ascii: Vec<u8> = raw.iter().map(|byte| byte & CPM_NAME_MASK).collect();
    String::from_utf8_lossy(&ascii).trim_end().to_string()
}

fn cpm_entry_filename(entry: &[u8]) -> String {
    let name = cpm_clean_name(&entry[CPM_NAME_OFFSET..CPM_NAME_OFFSET + CPM_NAME_LEN]);
    let ext = cpm_clean_name(&entry[CPM_EXT_OFFSET..CPM_EXT_OFFSET + CPM_EXT_LEN]);
    if ext.is_empty() {
        name
    } else {
        format!("{name}.{ext}")
    }
}

fn cpm_entry_extent(entry: &[u8]) -> usize {
    (entry[CPM_EXTENT_LOW_OFFSET] as usize & CPM_EXTENT_LOW_MASK)
        | (((entry[CPM_EXTENT_HIGH_OFFSET] as usize) << CPM_EXTENT_HIGH_SHIFT)
            & CPM_EXTENT_HIGH_MASK)
}

fn cpm_entry_blocks(entry: &[u8], block_num_16bit: bool) -> Vec<usize> {
    let mut blocks = Vec::new();
    if block_num_16bit {
        let mut idx = CPM_BLOCK_LIST_OFFSET;
        while idx + 1 < CPM_BLOCK_LIST_OFFSET + CPM_BLOCK_LIST_LEN {
            blocks.push(entry[idx] as usize | ((entry[idx + 1] as usize) << 8));
            idx += 2;
        }
    } else {
        for &byte in &entry[CPM_BLOCK_LIST_OFFSET..CPM_BLOCK_LIST_OFFSET + CPM_BLOCK_LIST_LEN] {
            blocks.push(byte as usize);
        }
    }
    blocks
}

struct CpmFile {
    user: u8,
    name: String,
    read_only: bool,
    extents: Vec<(usize, u8, Vec<usize>)>,
}

fn cpm_parse_directory(raw: &[u8], params: &CpmParams) -> Vec<CpmFile> {
    let data_offset = params.sys_bytes;
    let dir_len = params.dir_blocks * params.block_size;
    let start = data_offset.min(raw.len());
    let end = (data_offset + dir_len).min(raw.len());
    let directory = &raw[start..end];
    let mut files: Vec<CpmFile> = Vec::new();
    let mut pos = 0;
    while pos + CPM_DIR_ENTRY_LEN <= directory.len() {
        let entry = &directory[pos..pos + CPM_DIR_ENTRY_LEN];
        pos += CPM_DIR_ENTRY_LEN;
        let user = entry[0];
        if user > CPM_MAX_USER {
            continue;
        }
        let name = cpm_entry_filename(entry);
        let extent = cpm_entry_extent(entry);
        let record_count = entry[CPM_RECORD_COUNT_OFFSET];
        let read_only = (entry[CPM_EXT_OFFSET] & CPM_ATTR_BIT) != 0;
        let blocks: Vec<usize> = cpm_entry_blocks(entry, params.block_num_16bit)
            .into_iter()
            .filter(|&block| block != 0)
            .collect();
        if let Some(file) = files
            .iter_mut()
            .find(|file| file.user == user && file.name == name)
        {
            file.extents.push((extent, record_count, blocks));
        } else {
            files.push(CpmFile {
                user,
                name,
                read_only,
                extents: vec![(extent, record_count, blocks)],
            });
        }
    }
    files
}

fn cpm_ordered_blocks(file: &CpmFile) -> Vec<usize> {
    let mut extents = file.extents.clone();
    extents.sort_by_key(|extent| extent.0);
    extents
        .into_iter()
        .flat_map(|(_, _, blocks)| blocks)
        .collect()
}

fn cpm_assemble(file: &CpmFile, raw: &[u8], params: &CpmParams) -> Vec<u8> {
    let data_offset = params.sys_bytes;
    let mut extents = file.extents.clone();
    extents.sort_by_key(|extent| extent.0);
    let mut content = Vec::new();
    for (_extent, record_count, blocks) in &extents {
        let mut extent_bytes = Vec::new();
        for &block in blocks {
            let start = data_offset + block * params.block_size;
            if start < raw.len() {
                let end = (start + params.block_size).min(raw.len());
                extent_bytes.extend_from_slice(&raw[start..end]);
            }
        }
        let take = ((*record_count as usize) * CPM_RECORD_SIZE).min(extent_bytes.len());
        content.extend_from_slice(&extent_bytes[..take]);
    }
    content
}

fn cpm_split_name(name: &str) -> ([u8; CPM_NAME_LEN], [u8; CPM_EXT_LEN]) {
    let (stem, ext) = name.split_once('.').unwrap_or((name, ""));
    let mut name_bytes = [b' '; CPM_NAME_LEN];
    let mut ext_bytes = [b' '; CPM_EXT_LEN];
    for (slot, ch) in stem.bytes().take(CPM_NAME_LEN).enumerate() {
        name_bytes[slot] = ch.to_ascii_uppercase();
    }
    for (slot, ch) in ext.bytes().take(CPM_EXT_LEN).enumerate() {
        ext_bytes[slot] = ch.to_ascii_uppercase();
    }
    (name_bytes, ext_bytes)
}

fn cpm_build_entry(
    user: u8,
    name: &str,
    extent: usize,
    record_count: u8,
    blocks: &[usize],
    params: &CpmParams,
) -> [u8; CPM_DIR_ENTRY_LEN] {
    let mut entry = [0u8; CPM_DIR_ENTRY_LEN];
    entry[0] = user;
    let (name_bytes, ext_bytes) = cpm_split_name(name);
    entry[CPM_NAME_OFFSET..CPM_NAME_OFFSET + CPM_NAME_LEN].copy_from_slice(&name_bytes);
    entry[CPM_EXT_OFFSET..CPM_EXT_OFFSET + CPM_EXT_LEN].copy_from_slice(&ext_bytes);
    entry[CPM_EXTENT_LOW_OFFSET] = (extent & CPM_EXTENT_LOW_MASK) as u8;
    entry[CPM_EXTENT_HIGH_OFFSET] =
        ((extent >> CPM_EXTENT_HIGH_SHIFT) & CPM_EXTENT_HIGH_STORE_MASK) as u8;
    entry[CPM_RECORD_COUNT_OFFSET] = record_count;
    if params.block_num_16bit {
        for (slot, &block) in blocks.iter().enumerate() {
            entry[CPM_BLOCK_LIST_OFFSET + slot * 2] = (block & 0xFF) as u8;
            entry[CPM_BLOCK_LIST_OFFSET + slot * 2 + 1] = ((block >> 8) & 0xFF) as u8;
        }
    } else {
        for (slot, &block) in blocks.iter().enumerate() {
            entry[CPM_BLOCK_LIST_OFFSET + slot] = (block & 0xFF) as u8;
        }
    }
    entry
}

fn cpm_file_entries(
    user: u8,
    name: &str,
    records: usize,
    blocks: &[usize],
    params: &CpmParams,
) -> Vec<u8> {
    let mut out = Vec::new();
    let pointers_per_extent = cpm_block_pointer_count(params.block_num_16bit);
    let blocks_each = cpm_blocks_per_extent(params.block_size, params.block_num_16bit);
    let mut extent = 0usize;
    let mut consumed = 0usize;
    let mut remaining = records as isize;
    loop {
        let extent_records = remaining.clamp(0, CPM_RECORDS_PER_EXTENT as isize) as u8;
        let begin = consumed.min(blocks.len());
        let end = (consumed + blocks_each).min(blocks.len());
        let extent_blocks = &blocks[begin..end];
        let extent_blocks = &extent_blocks[..extent_blocks.len().min(pointers_per_extent)];
        out.extend_from_slice(&cpm_build_entry(
            user,
            name,
            extent,
            extent_records,
            extent_blocks,
            params,
        ));
        consumed += extent_blocks.len();
        remaining -= extent_records as isize;
        extent += 1;
        if remaining <= 0 {
            break;
        }
    }
    out
}

fn cpm_resolve(dir: &Path, user: u8, name: &str) -> PathBuf {
    if user == 0 {
        let flat = dir.join(name);
        if flat.exists() {
            return flat;
        }
    }
    dir.join(format!("{CPM_USER_DIR_PREFIX}{user:02}"))
        .join(name)
}

fn cpm_list_folder(dir: &Path) -> std::io::Result<Vec<(u8, String)>> {
    let mut top: Vec<String> = Vec::new();
    let mut subdirs: Vec<(u8, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if file_type.is_file() {
            if name != CPM_MANIFEST_NAME {
                top.push(name);
            }
        } else if file_type.is_dir()
            && let Some(rest) = name.strip_prefix(CPM_USER_DIR_PREFIX)
            && let Ok(user) = rest.parse::<u8>()
            && user <= CPM_MAX_USER
        {
            subdirs.push((user, entry.path()));
        }
    }
    top.sort_by_key(|name| name.to_lowercase());
    subdirs.sort_by_key(|(user, _)| *user);

    let mut files: Vec<(u8, String)> = top.into_iter().map(|name| (0, name)).collect();
    for (user, path) in subdirs {
        let mut names: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        names.sort_by_key(|name| name.to_lowercase());
        files.extend(names.into_iter().map(|name| (user, name)));
    }
    Ok(files)
}

fn cpm_to_bcd(value: u32) -> u8 {
    ((((value / 10) % 10) << 4) | (value % 10)) as u8
}

fn civil_from_unix(secs: i64) -> (i64, u32, u32, u32, u32) {
    let days = secs.div_euclid(SECONDS_PER_DAY);
    let rem = secs.rem_euclid(SECONDS_PER_DAY);
    let hour = (rem / SECONDS_PER_HOUR) as u32;
    let minute = ((rem % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE) as u32;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day, hour, minute)
}

fn cpm_write_stamp(buf: &mut [u8], offset: usize, time: std::io::Result<SystemTime>) {
    let Ok(time) = time else {
        return;
    };
    let Ok(dur) = time.duration_since(UNIX_EPOCH) else {
        return;
    };
    let (year, month, day, hour, minute) = civil_from_unix(dur.as_secs() as i64);
    if (CPM_DS_YEAR_MIN..CPM_DS_YEAR_MAX).contains(&year) && offset + CPM_DS_STAMP_LEN <= buf.len()
    {
        buf[offset] = cpm_to_bcd((year % 100) as u32);
        buf[offset + 1] = cpm_to_bcd(month);
        buf[offset + 2] = cpm_to_bcd(day);
        buf[offset + 3] = cpm_to_bcd(hour);
        buf[offset + 4] = cpm_to_bcd(minute);
    }
}

fn cpm_build_datestamp(dir_entries: usize, slot_files: &[(usize, PathBuf)]) -> Vec<u8> {
    let mut buf = vec![0u8; dir_entries * CPM_DS_RECORD_LEN];
    let mut signature = 0usize;
    let mut pos = CPM_DS_SIGNATURE_OFFSET;
    while pos < buf.len() {
        buf[pos] = CPM_DS_SIGNATURE[signature % CPM_DS_SIGNATURE.len()];
        signature += 1;
        pos += CPM_DS_RECORD_LEN;
    }
    for (slot, path) in slot_files {
        let base = slot * CPM_DS_RECORD_LEN;
        if base + CPM_DS_STAMP_COUNT * CPM_DS_STAMP_LEN <= buf.len()
            && let Ok(meta) = std::fs::metadata(path)
        {
            cpm_write_stamp(&mut buf, base, meta.created());
            cpm_write_stamp(&mut buf, base + CPM_DS_STAMP_LEN, meta.accessed());
            cpm_write_stamp(&mut buf, base + 2 * CPM_DS_STAMP_LEN, meta.modified());
        }
    }
    let mut pos = 0;
    while pos < buf.len() {
        let mut checksum = 0u32;
        let mut count = 0;
        while count < CPM_DS_CHECKSUM_SPAN && pos < buf.len() {
            checksum = checksum.wrapping_add(buf[pos] as u32);
            pos += 1;
            count += 1;
        }
        if pos < buf.len() {
            buf[pos] = checksum as u8;
            pos += 1;
        }
    }
    buf
}

fn cpm_from_bcd(value: u8) -> Option<u32> {
    let high = (value >> 4) as u32;
    let low = (value & 0x0F) as u32;
    if high > 9 || low > 9 {
        return None;
    }
    Some(high * 10 + low)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = (if month > 2 { month - 3 } else { month + 9 }) as i64;
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn cpm_stamp_to_unix(buf: &[u8], offset: usize) -> Option<i64> {
    if offset + CPM_DS_STAMP_LEN > buf.len() {
        return None;
    }
    let yy = cpm_from_bcd(buf[offset])? as i64;
    let month = cpm_from_bcd(buf[offset + 1])?;
    let day = cpm_from_bcd(buf[offset + 2])?;
    let hour = cpm_from_bcd(buf[offset + 3])?;
    let minute = cpm_from_bcd(buf[offset + 4])?;
    if month == 0 || month > 12 || day == 0 || day > 31 {
        return None;
    }
    let year = if yy >= CPM_DS_YEAR_MIN % 100 {
        1900 + yy
    } else {
        2000 + yy
    };
    Some(
        days_from_civil(year, month, day) * SECONDS_PER_DAY
            + hour as i64 * SECONDS_PER_HOUR
            + minute as i64 * SECONDS_PER_MINUTE,
    )
}

fn cpm_read_file_data(image: &[u8], file: &CpmFile, params: &CpmParams) -> Vec<u8> {
    let data_offset = params.sys_bytes;
    let mut out = Vec::new();
    for block in cpm_ordered_blocks(file) {
        let start = data_offset + block * params.block_size;
        if start >= image.len() {
            break;
        }
        let end = (start + params.block_size).min(image.len());
        out.extend_from_slice(&image[start..end]);
    }
    out
}

fn cpm_apply_datestamp(image: &[u8], dir: &Path, params: &CpmParams) {
    let files = cpm_parse_directory(image, params);
    let Some(ds_file) = files
        .iter()
        .find(|file| file.name.eq_ignore_ascii_case(CPM_DS_FILENAME))
    else {
        return;
    };
    let stamps = cpm_read_file_data(image, ds_file, params);
    let data_offset = params.sys_bytes;
    let dir_len = params.dir_blocks * params.block_size;
    let start = data_offset.min(image.len());
    let end = (data_offset + dir_len).min(image.len());
    let directory = &image[start..end];
    let mut pos = 0;
    let mut slot = 0;
    while pos + CPM_DIR_ENTRY_LEN <= directory.len() {
        let entry = &directory[pos..pos + CPM_DIR_ENTRY_LEN];
        pos += CPM_DIR_ENTRY_LEN;
        let user = entry[0];
        if user <= CPM_MAX_USER && cpm_entry_extent(entry) == 0 {
            let name = cpm_entry_filename(entry);
            if !name.eq_ignore_ascii_case(CPM_DS_FILENAME)
                && let Some(secs) =
                    cpm_stamp_to_unix(&stamps, slot * CPM_DS_RECORD_LEN + 2 * CPM_DS_STAMP_LEN)
            {
                let path = cpm_resolve(dir, user, &name);
                let mtime = filetime::FileTime::from_unix_time(secs, 0);
                let _ = filetime::set_file_mtime(&path, mtime);
            }
        }
        slot += 1;
    }
}

fn cpm_build_image(dir: &Path, format: DiskFormat, params: &CpmParams) -> std::io::Result<Vec<u8>> {
    let size = format.image_size();
    let mut image = vec![CPM_FILL_BYTE; size];
    let total_blocks = size / params.block_size;
    let data_offset = params.sys_bytes;
    let records_per_block = cpm_records_per_block(params.block_size);
    let dir_capacity = params.dir_blocks * params.block_size;
    let mut directory: Vec<u8> = Vec::new();
    let mut free_block = params.dir_blocks;
    let mut slot_files: Vec<(usize, PathBuf)> = Vec::new();
    let mut ds_blocks: Vec<usize> = Vec::new();

    if params.datestamp {
        let dir_entries = dir_capacity / CPM_DIR_ENTRY_LEN;
        let ds_records = (dir_entries * CPM_DS_RECORD_LEN).div_ceil(CPM_RECORD_SIZE);
        let ds_block_count = ds_records.div_ceil(records_per_block);
        if free_block + ds_block_count > total_blocks {
            return Err(std::io::Error::other("directory disk is full"));
        }
        ds_blocks = (free_block..free_block + ds_block_count).collect();
        free_block += ds_block_count;
        directory.extend_from_slice(&cpm_file_entries(
            0,
            CPM_DS_FILENAME,
            ds_records,
            &ds_blocks,
            params,
        ));
    }

    for (user, name) in cpm_list_folder(dir)? {
        if params.datestamp && name.eq_ignore_ascii_case(CPM_DS_FILENAME) {
            continue;
        }
        let path = cpm_resolve(dir, user, &name);
        let content = std::fs::read(&path)?;
        let records = content.len().div_ceil(CPM_RECORD_SIZE);
        let block_count = records.div_ceil(records_per_block);
        if free_block + block_count > total_blocks {
            return Err(std::io::Error::other("directory disk is full"));
        }
        let blocks: Vec<usize> = (free_block..free_block + block_count).collect();
        let start = data_offset + free_block * params.block_size;
        image[start..start + content.len()].copy_from_slice(&content);
        free_block += block_count;
        slot_files.push((directory.len() / CPM_DIR_ENTRY_LEN, path));
        directory.extend_from_slice(&cpm_file_entries(user, &name, records, &blocks, params));
    }

    if directory.len() > dir_capacity {
        return Err(std::io::Error::other(
            "too many files for the directory area",
        ));
    }
    image[data_offset..data_offset + directory.len()].copy_from_slice(&directory);

    if params.datestamp && !ds_blocks.is_empty() {
        let dir_entries = dir_capacity / CPM_DIR_ENTRY_LEN;
        let ds = cpm_build_datestamp(dir_entries, &slot_files);
        let start = data_offset + ds_blocks[0] * params.block_size;
        let end = (start + ds.len()).min(image.len());
        image[start..end].copy_from_slice(&ds[..end - start]);
    }
    Ok(image)
}

fn cpm_set_readonly(path: &Path, read_only: bool) {
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        if perms.readonly() != read_only {
            perms.set_readonly(read_only);
            let _ = std::fs::set_permissions(path, perms);
        }
    }
}

fn cpm_sync_folder(raw: &[u8], dir: &Path, params: &CpmParams) {
    use std::collections::HashSet;

    let files = cpm_parse_directory(raw, params);
    let mut wanted: HashSet<PathBuf> = HashSet::new();
    for file in &files {
        if params.datestamp && file.name.eq_ignore_ascii_case(CPM_DS_FILENAME) {
            continue;
        }
        let path = cpm_resolve(dir, file.user, &file.name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = cpm_assemble(file, raw, params);
        let differs = std::fs::read(&path)
            .map(|old| old != content)
            .unwrap_or(true);
        if differs {
            cpm_set_readonly(&path, false);
            let _ = std::fs::write(&path, &content);
        }
        cpm_set_readonly(&path, file.read_only);
        wanted.insert(path);
    }
    cpm_prune(dir, &wanted);
}

fn cpm_prune(dir: &Path, wanted: &std::collections::HashSet<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_file() {
            if entry.file_name().to_string_lossy() == CPM_MANIFEST_NAME {
                continue;
            }
            if !wanted.contains(&path) {
                let _ = std::fs::remove_file(&path);
            }
        } else if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_user_dir = name
                .strip_prefix(CPM_USER_DIR_PREFIX)
                .and_then(|rest| rest.parse::<u8>().ok())
                .is_some();
            if !is_user_dir {
                continue;
            }
            if let Ok(sub) = std::fs::read_dir(&path) {
                for child in sub.flatten() {
                    let child_path = child.path();
                    let is_file = child.file_type().map(|t| t.is_file()).unwrap_or(false);
                    if is_file && !wanted.contains(&child_path) {
                        let _ = std::fs::remove_file(&child_path);
                    }
                }
            }
        }
    }
}

pub struct DirectoryDisk {
    dir: PathBuf,
    format: DiskFormat,
    params: CpmParams,
    image: FloppyDisk,
    sys_sectors: usize,
    dir_sectors: usize,
    sides: usize,
    sectors_per_track: usize,
    sector_size: usize,
    sectors_per_block: usize,
    read_only: bool,
    auto_refresh: bool,
    last_build: Instant,
    last_write: Instant,
}

impl DirectoryDisk {
    pub fn open(dir: &Path, format: DiskFormat, writable: bool) -> std::io::Result<Self> {
        let params = cpm_params(format)
            .ok_or_else(|| std::io::Error::other("no CP/M layout for this disk format"))?;
        let raw = cpm_build_image(dir, format, &params)?;
        let image = format.to_disk(raw, !writable);
        let now = Instant::now();
        Ok(Self {
            dir: dir.to_path_buf(),
            format,
            sys_sectors: params.sys_bytes / format.sector_size,
            dir_sectors: params.dir_blocks * params.block_size / format.sector_size,
            sides: format.sides,
            sectors_per_track: format.sectors_per_track,
            sector_size: format.sector_size,
            sectors_per_block: params.block_size / format.sector_size,
            params,
            image,
            read_only: !writable,
            auto_refresh: true,
            last_build: now,
            last_write: now,
        })
    }

    fn abs_sector(&self, phys_cyl: usize, phys_head: usize, sector_num: u8) -> Option<usize> {
        if sector_num == 0 || phys_head >= self.sides {
            return None;
        }
        let logical = sector_num as usize - 1;
        if logical >= self.sectors_per_track {
            return None;
        }
        Some((phys_cyl * self.sides + phys_head) * self.sectors_per_track + logical)
    }

    fn in_directory(&self, abs: usize) -> bool {
        abs >= self.sys_sectors && abs < self.sys_sectors + self.dir_sectors
    }

    fn reconcile(&self) {
        cpm_sync_folder(&self.image.linear_bytes(), &self.dir, &self.params);
    }

    fn write_through_data(&self, abs: usize, sector_in_block: usize, data: &[u8]) {
        let raw = self.image.linear_bytes();
        let files = cpm_parse_directory(&raw, &self.params);
        let block_num = (abs - self.sys_sectors) / self.sectors_per_block;
        for file in &files {
            let Some(block_offset) = cpm_ordered_blocks(file)
                .iter()
                .position(|&block| block == block_num)
            else {
                continue;
            };
            let path = cpm_resolve(&self.dir, file.user, &file.name);
            if self.params.datestamp && file.name.eq_ignore_ascii_case(CPM_DS_FILENAME) {
                cpm_apply_datestamp(&raw, &self.dir, &self.params);
                return;
            }
            let offset =
                (block_offset * self.params.block_size + sector_in_block * self.sector_size) as u64;
            let within = std::fs::metadata(&path)
                .map(|meta| offset + data.len() as u64 <= meta.len())
                .unwrap_or(false);
            if within
                && let Ok(mut handle) = OpenOptions::new().write(true).open(&path)
                && handle.seek(SeekFrom::Start(offset)).is_ok()
            {
                let _ = handle.write_all(data);
                let _ = handle.flush();
            }
            return;
        }
    }

    fn maybe_refresh(&mut self, phys_cyl: usize, phys_head: usize, first_sector: bool) {
        if !self.auto_refresh || phys_cyl != 0 || phys_head != 0 || !first_sector {
            return;
        }
        let now = Instant::now();
        if now.duration_since(self.last_build) >= Duration::from_secs(DIR_REFRESH_SECONDS)
            && now.duration_since(self.last_write) >= Duration::from_millis(DIR_WRITE_GRACE_MILLIS)
        {
            self.rebuild();
        }
    }

    fn rebuild(&mut self) {
        if let Ok(raw) = cpm_build_image(&self.dir, self.format, &self.params) {
            self.image = self.format.to_disk(raw, self.read_only);
            self.last_build = Instant::now();
        }
    }
}

impl DiskBackend for DirectoryDisk {
    fn cylinders(&self) -> usize {
        self.image.cylinders()
    }

    fn sides(&self) -> usize {
        self.image.sides()
    }

    fn sectors_of_track(&self, phys_cyl: usize, phys_head: usize) -> usize {
        self.image.sectors_of_track(phys_cyl, phys_head)
    }

    fn is_hd(&self) -> bool {
        self.image.is_hd()
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }

    fn sector_by_index(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector_idx: usize,
    ) -> Option<Sector> {
        self.maybe_refresh(phys_cyl, phys_head, sector_idx == 0);
        self.image.sector_by_index(phys_cyl, phys_head, sector_idx)
    }

    fn sector_by_id(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        cyl: u8,
        head: u8,
        sector_num: u8,
        size_code: i16,
    ) -> Option<Sector> {
        self.maybe_refresh(phys_cyl, phys_head, sector_num == 1);
        self.image
            .sector_by_id(phys_cyl, phys_head, cyl, head, sector_num, size_code)
    }

    fn write_sector(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector: &Sector,
        data: &[u8],
        data_len: usize,
    ) -> bool {
        if self.read_only {
            return false;
        }
        if !self
            .image
            .write_sector(phys_cyl, phys_head, sector, data, data_len)
        {
            return false;
        }
        self.last_write = Instant::now();
        if let Some(abs) = self.abs_sector(phys_cyl, phys_head, sector.sector_num()) {
            if self.in_directory(abs) {
                self.reconcile();
            } else if abs >= self.sys_sectors + self.dir_sectors {
                let sector_in_block = (abs - self.sys_sectors) % self.sectors_per_block;
                self.write_through_data(abs, sector_in_block, &data[..data_len]);
            }
        }
        true
    }

    fn format_sector(
        &mut self,
        phys_cyl: usize,
        phys_head: usize,
        sector_num: u8,
        content: &[u8],
    ) -> bool {
        if self.read_only {
            return false;
        }
        if !self
            .image
            .format_sector(phys_cyl, phys_head, sector_num, content)
        {
            return false;
        }
        self.last_write = Instant::now();
        self.reconcile();
        true
    }
}

pub struct FloppyDiskDrive {
    disk: Option<Box<dyn DiskBackend>>,
    head: u8,
    present_cylinder: u16,
    new_cylinder: u16,
}

impl Default for FloppyDiskDrive {
    fn default() -> Self {
        Self::new()
    }
}

impl FloppyDiskDrive {
    pub fn new() -> Self {
        Self {
            disk: None,
            head: 0,
            present_cylinder: 0,
            new_cylinder: 0,
        }
    }

    pub fn reset(&mut self) {
        self.head = 0;
        self.present_cylinder = 0;
        self.new_cylinder = 0;
    }

    pub fn insert_disk(&mut self, disk: Box<dyn DiskBackend>) {
        self.disk = Some(disk);
    }

    pub fn remove_disk(&mut self) {
        self.disk = None;
    }

    pub fn disk(&self) -> Option<&dyn DiskBackend> {
        self.disk.as_deref()
    }

    pub fn is_ready(&self) -> bool {
        self.disk.is_some()
    }

    pub fn is_read_only(&self) -> bool {
        self.disk
            .as_ref()
            .map(|disk| disk.is_read_only())
            .unwrap_or(true)
    }

    pub fn cylinder(&self) -> u16 {
        self.present_cylinder
    }

    pub fn set_seek_mode(&mut self, head: u8, cyl: u16) {
        self.head = head;
        self.new_cylinder = cyl;
    }

    pub fn seek_step(&mut self) -> bool {
        if self.present_cylinder < self.new_cylinder {
            self.present_cylinder += 1;
        } else if self.present_cylinder > self.new_cylinder {
            self.present_cylinder -= 1;
        }
        self.present_cylinder == self.new_cylinder
    }

    pub fn read_sector_by_id(
        &mut self,
        phys_head: usize,
        start_idx: usize,
        cyl: u8,
        head: u8,
        sector_num: u8,
        size_code: u8,
    ) -> Option<Sector> {
        let phys_cyl = self.present_cylinder as usize;
        let disk = self.disk.as_mut()?;
        let mut sector =
            disk.sector_by_id(phys_cyl, phys_head, cyl, head, sector_num, size_code as i16);
        if let Some(found) = &sector
            && found.index_on_cylinder < start_idx
        {
            let mut idx = start_idx;
            sector = None;
            while let Some(candidate) = disk.sector_by_index(phys_cyl, phys_head, idx) {
                if candidate.cylinder == cyl
                    && candidate.head == head
                    && candidate.sector_num == sector_num
                    && candidate.size_code == size_code
                {
                    sector = Some(candidate);
                    break;
                }
                idx += 1;
            }
        }
        self.head = head;
        sector
    }

    pub fn read_sector_by_index(&mut self, phys_head: usize, sector_idx: usize) -> Option<Sector> {
        let phys_cyl = self.present_cylinder as usize;
        self.disk
            .as_mut()?
            .sector_by_index(phys_cyl, phys_head, sector_idx)
    }

    pub fn write_sector(
        &mut self,
        phys_head: usize,
        sector: &Sector,
        data: &[u8],
        data_len: usize,
    ) -> bool {
        let phys_cyl = self.present_cylinder as usize;
        match &mut self.disk {
            Some(disk) => disk.write_sector(phys_cyl, phys_head, sector, data, data_len),
            None => false,
        }
    }

    pub fn format_sector(&mut self, phys_head: usize, sector_num: u8, content: &[u8]) -> bool {
        let phys_cyl = self.present_cylinder as usize;
        match &mut self.disk {
            Some(disk) => disk.format_sector(phys_cyl, phys_head, sector_num, content),
            None => false,
        }
    }
}

#[derive(Debug)]
pub enum ContainerError {
    Truncated,
    Unsupported(String),
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerError::Truncated => write!(f, "disk image is truncated or malformed"),
            ContainerError::Unsupported(msg) => write!(f, "unsupported disk image: {msg}"),
        }
    }
}

impl std::error::Error for ContainerError {}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ImageKind {
    Raw,
    AnaDisk,
    CopyQm,
    CpcDsk,
    ImageDisk,
    TeleDisk,
}

fn detect_kind(bytes: &[u8], ext: Option<&str>) -> ImageKind {
    if bytes.len() >= COPYQM_MAGIC.len() && bytes[..COPYQM_MAGIC.len()] == COPYQM_MAGIC {
        return ImageKind::CopyQm;
    }
    if bytes.starts_with(IMAGEDISK_MAGIC) {
        return ImageKind::ImageDisk;
    }
    if bytes.starts_with(CPC_HEADER_STD) || bytes.starts_with(CPC_HEADER_EXT) {
        return ImageKind::CpcDsk;
    }
    if bytes.len() >= TELEDISK_MAGIC_PLAIN.len()
        && (bytes[..2] == TELEDISK_MAGIC_PLAIN || bytes[..2] == TELEDISK_MAGIC_PACKED)
    {
        return ImageKind::TeleDisk;
    }
    match ext.map(str::to_ascii_lowercase).as_deref() {
        Some("dump" | "anadisk" | "adl") => ImageKind::AnaDisk,
        _ => ImageKind::Raw,
    }
}

#[derive(Clone)]
pub struct DiskMount {
    pub path: PathBuf,
    pub format: DiskFormat,
    pub writable: bool,
}

pub fn mount(spec: &DiskMount) -> Result<Box<dyn DiskBackend>, ContainerError> {
    if spec.path.is_dir() {
        let disk = DirectoryDisk::open(&spec.path, spec.format, spec.writable).map_err(|err| {
            ContainerError::Unsupported(format!("{}: {err}", spec.path.display()))
        })?;
        return Ok(Box::new(disk));
    }
    let raw = std::fs::read(&spec.path)
        .map_err(|err| ContainerError::Unsupported(format!("{}: {err}", spec.path.display())))?;
    let ext = spec.path.extension().and_then(|ext| ext.to_str());
    if spec.writable {
        if raw.len() >= GZIP_MAGIC.len() && raw[..GZIP_MAGIC.len()] == GZIP_MAGIC {
            return Err(ContainerError::Unsupported(
                "gzipped images cannot be mounted writable; decompress it first".into(),
            ));
        }
        let open_err = |err: std::io::Error| {
            ContainerError::Unsupported(format!("{}: {err}", spec.path.display()))
        };
        match detect_kind(&raw, ext) {
            ImageKind::Raw => {
                let disk = FileImageDisk::open_raw(&spec.path, spec.format).map_err(open_err)?;
                Ok(Box::new(disk))
            }
            ImageKind::AnaDisk => {
                let image = parse_anadisk(&raw, false)?;
                let disk = FileImageDisk::open_container(&spec.path, image).map_err(open_err)?;
                Ok(Box::new(disk))
            }
            ImageKind::CpcDsk => {
                let image = parse_cpcdisk(&raw, false)?;
                let disk = FileImageDisk::open_container(&spec.path, image).map_err(open_err)?;
                Ok(Box::new(disk))
            }
            _ => Err(ContainerError::Unsupported(
                "writable mounts support raw, CPC and AnaDisk images; \
                 convert other containers with utils/disk2img.py"
                    .into(),
            )),
        }
    } else {
        load_image(raw, ext, spec.format, true)
    }
}

pub fn load_image(
    bytes: Vec<u8>,
    ext: Option<&str>,
    raw_format: DiskFormat,
    read_only: bool,
) -> Result<Box<dyn DiskBackend>, ContainerError> {
    let bytes = maybe_gunzip(bytes);
    let disk: FloppyDisk = match detect_kind(&bytes, ext) {
        ImageKind::Raw => raw_format.to_disk(bytes, read_only),
        ImageKind::AnaDisk => parse_anadisk(&bytes, read_only)?,
        ImageKind::CopyQm => parse_copyqm(&bytes, read_only)?,
        ImageKind::CpcDsk => parse_cpcdisk(&bytes, read_only)?,
        ImageKind::ImageDisk => parse_imagedisk(&bytes, read_only)?,
        ImageKind::TeleDisk => parse_teledisk(&bytes, read_only)?,
    };
    Ok(Box::new(disk))
}

fn maybe_gunzip(bytes: Vec<u8>) -> Vec<u8> {
    if bytes.len() >= GZIP_MAGIC.len() && bytes[..GZIP_MAGIC.len()] == GZIP_MAGIC {
        let mut decoded = Vec::new();
        if GzDecoder::new(bytes.as_slice())
            .read_to_end(&mut decoded)
            .is_ok()
        {
            return decoded;
        }
    }
    bytes
}

struct ByteReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn next(&mut self) -> Option<u8> {
        let value = self.bytes.get(self.pos).copied();
        if value.is_some() {
            self.pos += 1;
        }
        value
    }

    fn byte(&mut self) -> Result<u8, ContainerError> {
        self.next().ok_or(ContainerError::Truncated)
    }

    fn word_le(&mut self) -> Result<usize, ContainerError> {
        let low = self.byte()? as usize;
        let high = self.byte()? as usize;
        Ok((high << 8) | low)
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], ContainerError> {
        let end = self.pos.checked_add(len).ok_or(ContainerError::Truncated)?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or(ContainerError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }
}

struct DiskBuilder {
    tracks: BTreeMap<(usize, usize), Vec<StoredSector>>,
    max_cylinder: usize,
    max_head: usize,
    primary_sectors_per_track: usize,
    primary_sector_size: usize,
}

impl DiskBuilder {
    fn new() -> Self {
        Self {
            tracks: BTreeMap::new(),
            max_cylinder: 0,
            max_head: 0,
            primary_sectors_per_track: 0,
            primary_sector_size: 0,
        }
    }

    fn add(&mut self, phys_cyl: usize, phys_head: usize, sector: StoredSector) {
        self.max_cylinder = self.max_cylinder.max(phys_cyl);
        self.max_head = self.max_head.max(phys_head);
        self.primary_sector_size = self.primary_sector_size.max(sector.data.len());
        let track = self.tracks.entry((phys_cyl, phys_head)).or_default();
        track.push(sector);
        self.primary_sectors_per_track = self.primary_sectors_per_track.max(track.len());
    }

    fn finish(self, read_only: bool) -> Result<FloppyDisk, ContainerError> {
        if self.tracks.is_empty() {
            return Err(ContainerError::Truncated);
        }
        let cylinders = self.max_cylinder + 1;
        let sides = (self.max_head + 1).min(TWO_SIDES);
        let mut tracks = vec![Vec::new(); cylinders * sides];
        for ((cyl, head), sectors) in self.tracks {
            if cyl < cylinders && head < sides {
                tracks[cyl * sides + head] = sectors;
            }
        }
        Ok(FloppyDisk::from_tracks(
            tracks,
            cylinders,
            sides,
            self.primary_sectors_per_track,
            self.primary_sector_size,
            read_only,
        ))
    }
}

fn sized_buffer(size_code: u8, raw: &[u8]) -> Vec<u8> {
    let len = size_by_size_code(size_code);
    let mut data = vec![0u8; len];
    let copy = raw.len().min(len);
    data[..copy].copy_from_slice(&raw[..copy]);
    data
}

fn parse_anadisk(bytes: &[u8], read_only: bool) -> Result<FloppyDisk, ContainerError> {
    let mut reader = ByteReader::new(bytes);
    let mut builder = DiskBuilder::new();
    while let Some(phys_cyl) = reader.next() {
        let phys_head = reader.byte()?;
        let id_cyl = reader.byte()?;
        let id_head = reader.byte()?;
        let id_record = reader.byte()?;
        let id_size_code = reader.byte()?;
        let data_len = reader.word_le()?;
        if phys_head > 1 || id_head > 1 || id_record < 1 || id_size_code > 3 {
            break;
        }
        let nominal = size_by_size_code(id_size_code);
        let mut data = vec![0u8; nominal];
        let mut file_offset = None;
        if data_len > 0 {
            let data_pos = reader.position();
            let raw = reader.take(data_len)?;
            let copy = raw.len().min(data.len());
            data[..copy].copy_from_slice(&raw[..copy]);
            if data_len == nominal {
                file_offset = Some(data_pos as u64);
            }
        }
        let mut sector = StoredSector::new(id_cyl, id_head, id_record, id_size_code, data);
        if let Some(offset) = file_offset {
            sector = sector.with_offset(offset);
        }
        builder.add(phys_cyl as usize, phys_head as usize, sector);
    }
    builder.finish(read_only)
}

fn parse_imagedisk(bytes: &[u8], read_only: bool) -> Result<FloppyDisk, ContainerError> {
    let mut reader = ByteReader::new(bytes);
    while reader.byte()? != IMD_END_OF_COMMENT {}
    let mut builder = DiskBuilder::new();
    while let Some(_transfer_rate) = reader.next() {
        let cyl = reader.byte()? as usize;
        let head_flags = reader.byte()?;
        let sector_count = reader.byte()? as usize;
        let size_code = reader.byte()?;
        if size_code > SIZE_CODE_MAX {
            return Err(ContainerError::Unsupported(format!(
                "IMD sector size code {size_code}"
            )));
        }
        let sector_size = size_by_size_code(size_code);

        let sector_nums = reader.take(sector_count)?.to_vec();
        let sector_cyls = if head_flags & IMD_HEAD_CYL_MAP != 0 {
            Some(reader.take(sector_count)?.to_vec())
        } else {
            None
        };
        let sector_heads = if head_flags & IMD_HEAD_HEAD_MAP != 0 {
            Some(reader.take(sector_count)?.to_vec())
        } else {
            None
        };
        let phys_head = (head_flags & IMD_HEAD_NUMBER_MASK) as usize;

        for (i, &sector_num) in sector_nums.iter().enumerate() {
            let sector_type = reader.byte()?;
            let mut data = match sector_type {
                0 => vec![0u8; sector_size],
                1 | 3 | 5 | 7 => reader.take(sector_size)?.to_vec(),
                2 | 4 | 6 | 8 => vec![reader.byte()?; sector_size],
                other => {
                    return Err(ContainerError::Unsupported(format!(
                        "IMD sector record type {other}"
                    )));
                }
            };
            data.resize(sector_size, 0);
            let deleted = matches!(sector_type, 3 | 4 | 7 | 8);
            let error = (5..=8).contains(&sector_type);
            let id_cyl = sector_cyls
                .as_ref()
                .and_then(|m| m.get(i))
                .copied()
                .unwrap_or(cyl as u8);
            let id_head = sector_heads
                .as_ref()
                .and_then(|m| m.get(i))
                .copied()
                .unwrap_or(phys_head as u8);
            let mut sector = StoredSector::new(id_cyl, id_head, sector_num, size_code, data);
            sector.deleted = deleted;
            sector.error = error;
            builder.add(cyl, phys_head, sector);
        }
    }
    builder.finish(read_only)
}

fn parse_cpcdisk(bytes: &[u8], read_only: bool) -> Result<FloppyDisk, ContainerError> {
    if bytes.len() < CPC_FILE_HEADER_LEN {
        return Err(ContainerError::Truncated);
    }
    let header = &bytes[..CPC_FILE_HEADER_LEN];
    let extended = header.starts_with(CPC_HEADER_EXT);
    let cyls = header[CPC_CYL_COUNT_OFFSET] as usize;
    let sides = header[CPC_SIDE_COUNT_OFFSET] as usize;
    let std_track_size = (header[CPC_STD_TRACK_SIZE_OFFSET] as usize)
        | ((header[CPC_STD_TRACK_SIZE_OFFSET + 1] as usize) << 8);

    let mut reader = ByteReader::new(bytes);
    reader.take(CPC_FILE_HEADER_LEN)?;
    let mut builder = DiskBuilder::new();

    for track_index in 0..(cyls * sides) {
        let track_header = match reader.take(CPC_TRACK_HEADER_LEN) {
            Ok(slice) => slice,
            Err(_) => break,
        };
        if !track_header.starts_with(CPC_TRACK_HEADER) {
            break;
        }
        let track_size = if extended {
            let entry = CPC_EXT_TRACK_SIZE_TABLE + track_index;
            if entry >= CPC_FILE_HEADER_LEN {
                break;
            }
            (header[entry] as usize) << 8
        } else {
            std_track_size
        };
        let cyl = track_header[CPC_TRACK_CYL_OFFSET] as usize;
        let side = track_header[CPC_TRACK_SIDE_OFFSET] as usize;
        let track_size_code = track_header[CPC_TRACK_SIZE_CODE_OFFSET];
        if track_size_code > SIZE_CODE_MAX {
            break;
        }
        let sector_count = track_header[CPC_TRACK_SECTOR_COUNT_OFFSET] as usize;

        let track_data_start = reader.position();
        let track_buf = if track_size > CPC_TRACK_HEADER_LEN {
            reader.take(track_size - CPC_TRACK_HEADER_LEN)?
        } else {
            &[][..]
        };

        let mut info_pos = CPC_TRACK_SECTOR_LIST_OFFSET;
        let mut data_pos = 0usize;
        for _ in 0..sector_count {
            if info_pos + CPC_SECTOR_INFO_LEN > track_header.len() {
                break;
            }
            let id_cyl = track_header[info_pos];
            let id_head = track_header[info_pos + CPC_SECTOR_HEAD_OFFSET];
            let id_record = track_header[info_pos + CPC_SECTOR_ID_OFFSET];
            let id_size_code = track_header[info_pos + CPC_SECTOR_SIZE_CODE_OFFSET];
            let stored_len = if extended {
                (track_header[info_pos + CPC_SECTOR_ACTUAL_LEN_OFFSET] as usize)
                    | ((track_header[info_pos + CPC_SECTOR_ACTUAL_LEN_OFFSET + 1] as usize) << 8)
            } else if !extended && track_size_code == SIZE_CODE_MAX {
                CPC_BIG_SECTOR_SIZE
            } else {
                size_by_size_code(track_size_code)
            };
            info_pos += CPC_SECTOR_INFO_LEN;

            let available = track_buf.len().saturating_sub(data_pos);
            let copy = stored_len.min(available);
            let nominal = size_by_size_code(id_size_code);
            let mut data = vec![0u8; nominal.max(copy)];
            data[..copy].copy_from_slice(&track_buf[data_pos..data_pos + copy]);
            let file_offset = if stored_len == nominal && available >= stored_len {
                Some((track_data_start + data_pos) as u64)
            } else {
                None
            };
            data_pos += stored_len;

            let mut sector = StoredSector::new(id_cyl, id_head, id_record, id_size_code, data);
            if let Some(offset) = file_offset {
                sector = sector.with_offset(offset);
            }
            builder.add(cyl, side, sector);
        }
    }
    builder.finish(read_only)
}

fn parse_copyqm(bytes: &[u8], read_only: bool) -> Result<FloppyDisk, ContainerError> {
    if bytes.len() < COPYQM_HEADER_LEN {
        return Err(ContainerError::Truncated);
    }
    let header = &bytes[..COPYQM_HEADER_LEN];
    let word = |offset: usize| (header[offset] as usize) | ((header[offset + 1] as usize) << 8);

    let sector_size = word(COPYQM_SECTOR_SIZE_OFFSET);
    let sectors_per_track = word(COPYQM_SECTORS_PER_TRACK_OFFSET);
    let sides = (header[COPYQM_SIDES_OFFSET] as usize).clamp(1, TWO_SIDES);
    let used_cyls = header[COPYQM_USED_CYLS_OFFSET] as usize;
    let total_cyls = (header[COPYQM_CYLS_OFFSET] as usize).max(used_cyls);
    let sector_offset = header[COPYQM_SECTOR_OFFSET_OFFSET] as usize;
    let comment_len = word(COPYQM_COMMENT_LEN_OFFSET);
    if sector_size == 0 || sectors_per_track == 0 || total_cyls == 0 {
        return Err(ContainerError::Unsupported("CopyQM geometry".into()));
    }

    let mut reader = ByteReader::new(bytes);
    reader.take(COPYQM_HEADER_LEN)?;
    reader.take(comment_len)?;

    let disk_size = total_cyls * sides * sectors_per_track * sector_size;
    if disk_size > MAX_IMAGE_SIZE {
        return Err(ContainerError::Unsupported("CopyQM image size".into()));
    }
    let mut disk_bytes = vec![0u8; disk_size];
    let mut dst = 0usize;
    while dst < disk_size {
        let Ok(len) = reader.word_le() else { break };
        if len & COPYQM_RUN_FLAG != 0 {
            let run = (COPYQM_RUN_MODULO - len) & COPYQM_WORD_MASK;
            let Ok(value) = reader.byte() else { break };
            let mut n = run;
            while dst < disk_size && n > 0 {
                disk_bytes[dst] = value;
                dst += 1;
                n -= 1;
            }
        } else {
            let mut n = len;
            while dst < disk_size && n > 0 {
                let Ok(value) = reader.byte() else { break };
                disk_bytes[dst] = value;
                dst += 1;
                n -= 1;
            }
        }
    }

    let size_code = size_code_by_size(sector_size);
    let mut builder = DiskBuilder::new();
    let mut block = 0usize;
    for cyl in 0..total_cyls {
        for head in 0..sides {
            for sector_idx in 0..sectors_per_track {
                let start = block * sector_size;
                let data = sized_buffer(size_code, &disk_bytes[start..start + sector_size]);
                builder.add(
                    cyl,
                    head,
                    StoredSector::new(
                        cyl as u8,
                        head as u8,
                        (sector_idx + 1 + sector_offset) as u8,
                        size_code,
                        data,
                    ),
                );
                block += 1;
            }
        }
    }
    builder.finish(read_only)
}

fn parse_teledisk(bytes: &[u8], read_only: bool) -> Result<FloppyDisk, ContainerError> {
    if bytes.len() < TELEDISK_HEADER_LEN {
        return Err(ContainerError::Truncated);
    }
    let signature = [bytes[0], bytes[1]];
    let compressed = signature == TELEDISK_MAGIC_PACKED;
    if !compressed && signature != TELEDISK_MAGIC_PLAIN {
        return Err(ContainerError::Truncated);
    }
    let version = bytes[TELEDISK_VERSION_OFFSET];
    if version != TELEDISK_SUPPORTED_VERSION {
        return Err(ContainerError::Unsupported(format!(
            "TeleDisk format version {version:#04X}"
        )));
    }
    let has_remark = bytes[TELEDISK_STEPPING_OFFSET] & TELEDISK_REMARK_FLAG != 0;

    let body = if compressed {
        lzhuf::decompress(&bytes[TELEDISK_HEADER_LEN..], MAX_IMAGE_SIZE)
    } else {
        bytes[TELEDISK_HEADER_LEN..].to_vec()
    };
    let mut reader = ByteReader::new(&body);

    if has_remark {
        reader.word_le()?;
        let comment_len = reader.word_le()?;
        reader.take(6)?;
        reader.take(comment_len)?;
    }

    let mut builder = DiskBuilder::new();
    while let Some(sector_count) = reader.next() {
        if sector_count == TELEDISK_SECTOR_PHANTOM {
            break;
        }
        let Some(track) = reader.next() else {
            break;
        };
        let Some(head) = reader.next() else {
            break;
        };
        if reader.next().is_none() {
            break;
        }
        let phys_head = (head & TELEDISK_HEAD_NUMBER_MASK) as usize;
        for _ in 0..sector_count {
            let sec_track = reader.byte()?;
            let sec_head = reader.byte()?;
            let sec_num = reader.byte()?;
            let sec_size_code = reader.byte()?;
            let sec_ctrl = reader.byte()?;
            reader.byte()?;
            if sec_size_code > SIZE_CODE_MAX {
                return Err(ContainerError::Unsupported(format!(
                    "TeleDisk sector size code {sec_size_code}"
                )));
            }
            let sector_size = size_by_size_code(sec_size_code);
            let mut data = vec![0u8; sector_size];
            if sec_ctrl & TELEDISK_SECTOR_NO_DATA_MASK == 0 {
                decode_teledisk_data(&mut reader, &mut data)?;
            }
            let bogus_header = sec_ctrl & TELEDISK_SECTOR_BOGUS_HEADER != 0;
            let id_cyl = if bogus_header { track } else { sec_track };
            let id_head = if bogus_header {
                head & TELEDISK_HEAD_NUMBER_MASK
            } else {
                sec_head
            };
            let mut sector = StoredSector::new(id_cyl, id_head, sec_num, sec_size_code, data);
            sector.deleted = sec_ctrl & TELEDISK_SECTOR_DELETED != 0;
            sector.error = sec_ctrl & TELEDISK_SECTOR_CRC_ERROR != 0;
            builder.add(track as usize, phys_head, sector);
        }
    }
    builder.finish(read_only)
}

fn decode_teledisk_data(
    reader: &mut ByteReader<'_>,
    data: &mut [u8],
) -> Result<(), ContainerError> {
    let mut len = reader.word_le()?;
    if len == 0 {
        return Ok(());
    }
    let encoding = reader.byte()?;
    len -= 1;
    let mut pos = 0usize;
    let mut put = |pos: &mut usize, value: u8| {
        if *pos < data.len() {
            data[*pos] = value;
            *pos += 1;
        }
    };
    match encoding {
        0 => {
            while len > 0 {
                let value = reader.byte()?;
                put(&mut pos, value);
                len -= 1;
            }
        }
        1 => {
            if len >= 4 {
                let mut n = reader.word_le()?;
                let b0 = reader.byte()?;
                let b1 = reader.byte()?;
                len -= 4;
                while n > 0 {
                    put(&mut pos, b0);
                    put(&mut pos, b1);
                    n -= 1;
                }
            }
        }
        2 => {
            while len >= 2 {
                let kind = reader.byte()?;
                let count = reader.byte()? as usize;
                len -= 2;
                match kind {
                    0 => {
                        let mut n = count;
                        while len > 0 && n > 0 {
                            let value = reader.byte()?;
                            put(&mut pos, value);
                            n -= 1;
                            len -= 1;
                        }
                    }
                    1 => {
                        if len >= 2 {
                            let b0 = reader.byte()?;
                            let b1 = reader.byte()?;
                            len -= 2;
                            let mut n = count;
                            while n > 0 {
                                put(&mut pos, b0);
                                put(&mut pos, b1);
                                n -= 1;
                            }
                        }
                    }
                    other => {
                        return Err(ContainerError::Unsupported(format!(
                            "TeleDisk sub-encoding {other}"
                        )));
                    }
                }
            }
        }
        other => {
            return Err(ContainerError::Unsupported(format!(
                "TeleDisk sector encoding {other:#04X}"
            )));
        }
    }
    while len > 0 {
        reader.byte()?;
        len -= 1;
    }
    Ok(())
}
