use crate::interrupt;
use crate::mmio::reg8;
#[cfg(feature = "chip-8258")]
use crate::regs8258::{REG_MSPI_CTRL, REG_MSPI_DATA};

// Bit values mirror SDK semantics used by vendor flash code.
const FLD_MSPI_CS: u8 = 1 << 0;
const FLD_MSPI_BUSY: u8 = 1 << 4;

pub const PAGE_SIZE: usize = 256;
pub const SECTOR_SIZE: usize = 4096;

const FLASH_WRITE_CMD: u8 = 0x02;
const FLASH_READ_CMD: u8 = 0x03;
const FLASH_SECT_ERASE_CMD: u8 = 0x20;
const FLASH_GET_JEDEC_ID: u8 = 0x9f;
const FLASH_READ_UID_CMD_GD_PUYA_ZB_TH: u8 = 0x4b;
const FLASH_READ_UID_CMD_XTX: u8 = 0x5a;
const FLASH_WRITE_STATUS_CMD_LOWBYTE: u8 = 0x01;
const FLASH_READ_STATUS_CMD_LOWBYTE: u8 = 0x05;
const FLASH_WRITE_ENABLE_CMD: u8 = 0x06;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlashError {
    UnalignedSectorAddress(u32),
    CrossesPageBoundary { addr: u32, len: usize },
    UnsupportedUidCommand(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlashStatusKind {
    Status8Bit,
    Status16BitSingleCommand,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Flash;

impl Flash {
    #[inline(always)]
    pub const fn new() -> Self {
        Self
    }

    #[inline(always)]
    pub fn read_page(&self, addr: u32, data: &mut [u8]) {
        read_page(addr, data);
    }

    #[inline(always)]
    pub fn write_page(&self, addr: u32, data: &[u8]) {
        write_page(addr, data);
    }

    #[inline(always)]
    pub fn write_page_aligned(&self, addr: u32, data: &[u8]) -> Result<(), FlashError> {
        write_page_aligned(addr, data)
    }

    #[inline(always)]
    pub fn erase_sector(&self, addr: u32) {
        erase_sector(addr);
    }

    #[inline(always)]
    pub fn erase_sector_checked(&self, addr: u32) -> Result<(), FlashError> {
        erase_sector_checked(addr)
    }

    #[inline(always)]
    pub fn read_status(&self, cmd: u8) -> u8 {
        read_status(cmd)
    }

    #[inline(always)]
    pub fn write_status(&self, kind: FlashStatusKind, data: u16) {
        write_status(kind, data);
    }

    #[inline(always)]
    pub fn read_raw_mid(&self) -> u32 {
        read_raw_mid()
    }

    #[inline(always)]
    pub fn read_mid(&self) -> u32 {
        read_mid()
    }

    #[inline(always)]
    pub fn read_jedec_id(&self) -> FlashMid {
        read_jedec_id()
    }

    #[inline(always)]
    pub fn read_uid(&self, cmd: u8, uid: &mut [u8; 16]) -> Result<(), FlashError> {
        read_uid(cmd, uid)
    }

    #[inline(always)]
    pub fn read_uid_default(&self, uid: &mut [u8; 16]) -> Result<(), FlashError> {
        read_uid_default(uid)
    }

    #[inline(always)]
    pub fn read_vdd_f_calibration_value(&self) -> u8 {
        read_vdd_f_calibration_value()
    }

    #[inline(always)]
    pub fn is_zb(&self) -> bool {
        flash_is_zb() != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlashVendor {
    Zbit,
    GigaDevice,
    Puya,
    TongHeng,
    Th,
    Unknown(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlashCapacity {
    K64,
    K128,
    K256,
    K512,
    M1,
    M2,
    M4,
    M8,
    Unknown(u8),
}

impl FlashCapacity {
    #[inline(always)]
    pub const fn bytes(self) -> Option<usize> {
        match self {
            Self::K64 => Some(64 * 1024),
            Self::K128 => Some(128 * 1024),
            Self::K256 => Some(256 * 1024),
            Self::K512 => Some(512 * 1024),
            Self::M1 => Some(1024 * 1024),
            Self::M2 => Some(2 * 1024 * 1024),
            Self::M4 => Some(4 * 1024 * 1024),
            Self::M8 => Some(8 * 1024 * 1024),
            Self::Unknown(_) => None,
        }
    }

    #[inline(always)]
    pub const fn vdd_f_calibration_addr(self) -> Option<u32> {
        match self {
            Self::K64 => Some(0x00e1c0),
            Self::K128 => Some(0x01e1c0),
            Self::K512 => Some(0x0771c0),
            Self::M1 => Some(0x0fe1c0),
            Self::M2 => Some(0x1fe1c0),
            Self::K256 | Self::M4 | Self::M8 | Self::Unknown(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FlashMid(u32);

impl FlashMid {
    #[inline(always)]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw & 0x00ff_ffff)
    }

    #[inline(always)]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub const fn manufacturer_id(self) -> u8 {
        self.0 as u8
    }

    #[inline(always)]
    pub const fn memory_type(self) -> u8 {
        (self.0 >> 8) as u8
    }

    #[inline(always)]
    pub const fn capacity_code(self) -> u8 {
        (self.0 >> 16) as u8
    }

    #[inline(always)]
    pub const fn capacity(self) -> FlashCapacity {
        match self.capacity_code() {
            0x10 => FlashCapacity::K64,
            0x11 => FlashCapacity::K128,
            0x12 => FlashCapacity::K256,
            0x13 => FlashCapacity::K512,
            0x14 => FlashCapacity::M1,
            0x15 => FlashCapacity::M2,
            0x16 => FlashCapacity::M4,
            0x17 => FlashCapacity::M8,
            other => FlashCapacity::Unknown(other),
        }
    }

    #[inline(always)]
    pub const fn vendor(self) -> FlashVendor {
        match self.raw() & 0x0000_ffff {
            0x325e => FlashVendor::Zbit,
            0x60c8 => FlashVendor::GigaDevice,
            0x4051 => FlashVendor::GigaDevice,
            0x6085 => FlashVendor::Puya,
            0x60eb => FlashVendor::TongHeng,
            0x60cd => FlashVendor::Th,
            raw => FlashVendor::Unknown(raw),
        }
    }

    #[inline(always)]
    pub const fn is_zbit(self) -> bool {
        matches!(self.vendor(), FlashVendor::Zbit)
    }
}

#[inline(always)]
fn mspi_wait() {
    unsafe {
        while (core::ptr::read_volatile(reg8(REG_MSPI_CTRL).cast_const()) & FLD_MSPI_BUSY) != 0 {}
    }
}

#[inline(always)]
fn mspi_high() {
    unsafe {
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), FLD_MSPI_CS);
    }
}

#[inline(always)]
fn mspi_low() {
    unsafe {
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), 0);
    }
}

#[inline(always)]
fn mspi_get() -> u8 {
    unsafe { core::ptr::read_volatile(reg8(REG_MSPI_DATA).cast_const()) }
}

#[inline(always)]
fn mspi_write(byte: u8) {
    unsafe {
        core::ptr::write_volatile(reg8(REG_MSPI_DATA), byte);
    }
}

#[inline(always)]
fn mspi_ctrl_write(byte: u8) {
    unsafe {
        core::ptr::write_volatile(reg8(REG_MSPI_CTRL), byte);
    }
}

#[inline(always)]
fn send_cmd(cmd: u8) {
    mspi_high();
    // Vendor code inserts sleep_us(1) here. For read-only helpers used after
    // startup, a few nops are sufficient to separate CS edges.
    for _ in 0..16 {
        core::hint::spin_loop();
    }
    mspi_low();
    mspi_write(cmd);
    mspi_wait();
}

#[inline(always)]
fn send_addr(addr: u32) {
    mspi_write((addr >> 16) as u8);
    mspi_wait();
    mspi_write((addr >> 8) as u8);
    mspi_wait();
    mspi_write(addr as u8);
    mspi_wait();
}

#[inline(always)]
fn wait_done() {
    crate::timer::clock_time();
    send_cmd(FLASH_READ_STATUS_CMD_LOWBYTE);
    for _ in 0..10_000_000 {
        if (mspi_read() & 0x01) == 0 {
            break;
        }
    }
    mspi_high();
}

#[inline(always)]
fn mspi_read() -> u8 {
    mspi_write(0);
    mspi_wait();
    mspi_get()
}

pub fn read_raw(cmd: u8, addr: u32, addr_en: bool, dummy_count: u8, data: &mut [u8]) {
    let irq_enabled = interrupt::disable();

    send_cmd(cmd);
    if addr_en {
        send_addr(addr);
    }
    for _ in 0..dummy_count {
        mspi_write(0);
        mspi_wait();
    }
    mspi_write(0);
    mspi_wait();
    mspi_ctrl_write(0x0a);
    mspi_wait();

    for byte in data {
        *byte = mspi_get();
        mspi_wait();
    }
    mspi_high();

    interrupt::restore(irq_enabled);
}

pub fn write_raw(cmd: u8, addr: u32, addr_en: bool, data: &[u8]) {
    let irq_enabled = interrupt::disable();

    send_cmd(FLASH_WRITE_ENABLE_CMD);
    send_cmd(cmd);
    if addr_en {
        send_addr(addr);
    }
    for &byte in data {
        mspi_write(byte);
        mspi_wait();
    }
    mspi_high();
    wait_done();

    interrupt::restore(irq_enabled);
}

#[inline(always)]
pub fn read_page(addr: u32, data: &mut [u8]) {
    read_raw(FLASH_READ_CMD, addr, true, 0, data);
}

#[inline(always)]
pub fn erase_sector(addr: u32) {
    write_raw(FLASH_SECT_ERASE_CMD, addr, true, &[]);
}

pub fn write_page(mut addr: u32, mut data: &[u8]) {
    let mut page_space = PAGE_SIZE - (addr as usize & (PAGE_SIZE - 1));

    while !data.is_empty() {
        let chunk_len = data.len().min(page_space);
        let (chunk, rest) = data.split_at(chunk_len);
        write_raw(FLASH_WRITE_CMD, addr, true, chunk);
        addr = addr.wrapping_add(chunk_len as u32);
        data = rest;
        page_space = PAGE_SIZE;
    }
}

#[inline(always)]
pub fn write_page_aligned(addr: u32, data: &[u8]) -> Result<(), FlashError> {
    let offset = addr as usize & (PAGE_SIZE - 1);
    if offset + data.len() > PAGE_SIZE {
        return Err(FlashError::CrossesPageBoundary { addr, len: data.len() });
    }
    write_page(addr, data);
    Ok(())
}

#[inline(always)]
pub fn read_status(cmd: u8) -> u8 {
    let mut status = [0u8; 1];
    read_raw(cmd, 0, false, 0, &mut status);
    status[0]
}

pub fn write_status(kind: FlashStatusKind, data: u16) {
    let buf = [data as u8, (data >> 8) as u8];
    match kind {
        FlashStatusKind::Status8Bit => write_raw(FLASH_WRITE_STATUS_CMD_LOWBYTE, 0, false, &buf[..1]),
        FlashStatusKind::Status16BitSingleCommand => {
            write_raw(FLASH_WRITE_STATUS_CMD_LOWBYTE, 0, false, &buf)
        }
    }
}

#[inline(always)]
pub fn read_raw_mid() -> u32 {
    let mut raw = [0u8; 4];
    read_raw(FLASH_GET_JEDEC_ID, 0, false, 0, &mut raw[..3]);
    u32::from_le_bytes(raw)
}

#[inline(always)]
pub fn read_mid() -> u32 {
    let flash_mid = read_raw_mid();
    if flash_mid == 0x1460c8 {
        let mut sfdp = [0u8; 4];
        read_raw(FLASH_READ_UID_CMD_XTX, 0, true, 1, &mut sfdp);
        if sfdp == *b"SFDP" {
            return 0x0114_60c8;
        }
    }
    flash_mid
}

#[inline(always)]
pub fn read_jedec_id() -> FlashMid {
    FlashMid::from_raw(read_mid())
}

pub fn read_uid(cmd: u8, uid: &mut [u8; 16]) -> Result<(), FlashError> {
    match cmd {
        FLASH_READ_UID_CMD_GD_PUYA_ZB_TH => {
            read_raw(cmd, 0, true, 1, uid);
            Ok(())
        }
        other => Err(FlashError::UnsupportedUidCommand(other)),
    }
}

#[inline(always)]
pub fn read_uid_default(uid: &mut [u8; 16]) -> Result<(), FlashError> {
    read_uid(FLASH_READ_UID_CMD_GD_PUYA_ZB_TH, uid)
}

#[inline(always)]
pub fn read_vdd_f_calibration_value() -> u8 {
    let mid = read_jedec_id();
    match mid.capacity().vdd_f_calibration_addr() {
        Some(addr) => {
            let mut value = [0u8; 1];
            read_page(addr, &mut value);
            value[0]
        }
        None => 0xff,
    }
}

#[inline(always)]
pub fn erase_sector_checked(addr: u32) -> Result<(), FlashError> {
    if (addr as usize & (SECTOR_SIZE - 1)) != 0 {
        return Err(FlashError::UnalignedSectorAddress(addr));
    }
    erase_sector(addr);
    Ok(())
}

#[unsafe(no_mangle)]
pub extern "C" fn flash_is_zb() -> u8 {
    let flash_mid = read_mid();
    if flash_mid == 0x13325e || flash_mid == 0x14325e {
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn flash_vdd_f_calib() {}
