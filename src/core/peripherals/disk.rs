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
use std::io::Read;

use flate2::read::GzDecoder;

const HEAD_MASK: usize = 0x01;
const SIZE_CODE_BASE: usize = 0x80;
const MAX_SIZE_CODE: u8 = 6;
const HD_SECTOR_SIZE: usize = 512;
const HD_SECTORS_PER_TRACK: usize = 15;

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
    while size < sector_size && value < MAX_SIZE_CODE {
        value += 1;
        size <<= 1;
    }
    value
}

fn size_by_size_code(size_code: u8) -> usize {
    SIZE_CODE_BASE << (size_code.min(MAX_SIZE_CODE) as usize)
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
        }
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

#[derive(Clone)]
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

    pub fn cylinders(&self) -> usize {
        self.cylinders
    }

    pub fn sides(&self) -> usize {
        self.sides
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn is_hd(&self) -> bool {
        self.primary_sectors_per_track >= HD_SECTORS_PER_TRACK
            && self.primary_sector_size <= HD_SECTOR_SIZE
    }

    fn track_index(&self, phys_cyl: usize, phys_head: usize) -> Option<usize> {
        let head = phys_head & HEAD_MASK;
        if phys_cyl >= self.cylinders || head >= self.sides {
            return None;
        }
        Some(phys_cyl * self.sides + head)
    }

    fn track(&self, phys_cyl: usize, phys_head: usize) -> Option<&Vec<StoredSector>> {
        self.track_index(phys_cyl, phys_head)
            .map(|idx| &self.tracks[idx])
    }

    pub fn sectors_of_track(&self, phys_cyl: usize, phys_head: usize) -> usize {
        self.track(phys_cyl, phys_head).map_or(0, Vec::len)
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

    pub fn sector_by_index(
        &self,
        phys_cyl: usize,
        phys_head: usize,
        sector_idx: usize,
    ) -> Option<Sector> {
        let track = self.track(phys_cyl, phys_head)?;
        track
            .get(sector_idx)
            .map(|stored| Self::build_sector(stored, sector_idx))
    }

    pub fn sector_by_id(
        &self,
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

    pub fn write_sector(
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

    pub fn format_sector(
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

pub struct FloppyDiskDrive {
    disk: Option<FloppyDisk>,
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

    pub fn insert_disk(&mut self, disk: FloppyDisk) {
        self.disk = Some(disk);
    }

    pub fn remove_disk(&mut self) {
        self.disk = None;
    }

    pub fn disk(&self) -> Option<&FloppyDisk> {
        self.disk.as_ref()
    }

    pub fn is_ready(&self) -> bool {
        self.disk.is_some()
    }

    pub fn is_read_only(&self) -> bool {
        self.disk
            .as_ref()
            .map(FloppyDisk::is_read_only)
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
        let disk = self.disk.as_ref()?;
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

    pub fn read_sector_by_index(&self, phys_head: usize, sector_idx: usize) -> Option<Sector> {
        self.disk
            .as_ref()?
            .sector_by_index(self.present_cylinder as usize, phys_head, sector_idx)
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

const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];
const COPYQM_MAGIC: [u8; 3] = [b'C', b'Q', 0x14];
const IMAGEDISK_MAGIC: &[u8] = b"IMD ";
const TELEDISK_MAGIC_PLAIN: [u8; 2] = [b'T', b'D'];
const TELEDISK_MAGIC_PACKED: [u8; 2] = [b't', b'd'];
const CPC_HEADER_STD: &[u8] = b"MV - CPCEMU";
const CPC_HEADER_EXT: &[u8] = b"EXTENDED CPC DSK";
const CPC_TRACK_HEADER: &[u8] = b"Track-Info\r\n";

const TWO_SIDES: usize = 2;
const SIZE_CODE_MAX: u8 = 6;
const MAX_IMAGE_SIZE: usize = 4 * 1024 * 1024;

const IMD_END_OF_COMMENT: u8 = 0x1A;
const IMD_HEAD_CYL_MAP: u8 = 0x80;
const IMD_HEAD_HEAD_MAP: u8 = 0x40;
const IMD_HEAD_NUMBER_MASK: u8 = 0x01;

const TELEDISK_SUPPORTED_VERSION: u8 = 0x15;
const TELEDISK_HEADER_LEN: usize = 12;
const TELEDISK_VERSION_OFFSET: usize = 4;
const TELEDISK_STEPPING_OFFSET: usize = 7;
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
const CPC_BIG_SECTOR_SIZE: usize = 0x1800;

const COPYQM_HEADER_LEN: usize = 133;
const COPYQM_SECTOR_SIZE_OFFSET: usize = 3;
const COPYQM_SECTORS_PER_TRACK_OFFSET: usize = 0x10;
const COPYQM_SIDES_OFFSET: usize = 0x12;
const COPYQM_USED_CYLS_OFFSET: usize = 0x5A;
const COPYQM_CYLS_OFFSET: usize = 0x5B;
const COPYQM_COMMENT_LEN_OFFSET: usize = 0x6F;
const COPYQM_SECTOR_OFFSET_OFFSET: usize = 0x71;

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

pub fn load_image(
    bytes: Vec<u8>,
    ext: Option<&str>,
    raw_format: DiskFormat,
    read_only: bool,
) -> Result<FloppyDisk, ContainerError> {
    let bytes = maybe_gunzip(bytes);
    match detect_kind(&bytes, ext) {
        ImageKind::Raw => Ok(raw_format.to_disk(bytes, read_only)),
        ImageKind::AnaDisk => parse_anadisk(&bytes, read_only),
        ImageKind::CopyQm => parse_copyqm(&bytes, read_only),
        ImageKind::CpcDsk => parse_cpcdisk(&bytes, read_only),
        ImageKind::ImageDisk => parse_imagedisk(&bytes, read_only),
        ImageKind::TeleDisk => parse_teledisk(&bytes, read_only),
    }
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
        let mut data = vec![0u8; size_by_size_code(id_size_code)];
        if data_len > 0 {
            let raw = reader.take(data_len)?;
            let copy = raw.len().min(data.len());
            data[..copy].copy_from_slice(&raw[..copy]);
        }
        builder.add(
            phys_cyl as usize,
            phys_head as usize,
            StoredSector::new(id_cyl, id_head, id_record, id_size_code, data),
        );
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
            let id_head = track_header[info_pos + 1];
            let id_record = track_header[info_pos + 2];
            let id_size_code = track_header[info_pos + 3];
            let stored_len = if extended {
                (track_header[info_pos + 6] as usize) | ((track_header[info_pos + 7] as usize) << 8)
            } else if !extended && track_size_code == SIZE_CODE_MAX {
                CPC_BIG_SECTOR_SIZE
            } else {
                size_by_size_code(track_size_code)
            };
            info_pos += CPC_SECTOR_INFO_LEN;

            let available = track_buf.len().saturating_sub(data_pos);
            let copy = stored_len.min(available);
            let mut data = vec![0u8; size_by_size_code(id_size_code).max(copy)];
            data[..copy].copy_from_slice(&track_buf[data_pos..data_pos + copy]);
            data_pos += stored_len;

            builder.add(
                cyl,
                side,
                StoredSector::new(id_cyl, id_head, id_record, id_size_code, data),
            );
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
        if len & 0x8000 != 0 {
            let run = (0x10000 - len) & 0xFFFF;
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
        reader.word_le()?; // comment crc
        let comment_len = reader.word_le()?;
        reader.take(6)?; // year month day hour minute second
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
            break; // track header crc
        }
        let phys_head = (head & 0x01) as usize;
        for _ in 0..sector_count {
            let sec_track = reader.byte()?;
            let sec_head = reader.byte()?;
            let sec_num = reader.byte()?;
            let sec_size_code = reader.byte()?;
            let sec_ctrl = reader.byte()?;
            reader.byte()?; // sector crc
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
            let id_head = if bogus_header { head & 0x01 } else { sec_head };
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
