use crate::interrupt;

const MSPI_DATA_ADDR: usize = 0x0080_000c;
const MSPI_CTRL_ADDR: usize = 0x0080_000d;

const FLD_MSPI_BUSY: u8 = 1 << 0;
const FLD_MSPI_CS: u8 = 1 << 1;

pub const PAGE_SIZE: usize = 256;
pub const SECTOR_SIZE: usize = 4096;

const FLASH_WRITE_CMD: u8 = 0x02;
const FLASH_READ_CMD: u8 = 0x03;
const FLASH_SECT_ERASE_CMD: u8 = 0x20;
const FLASH_GET_JEDEC_ID: u8 = 0x9f;
const FLASH_WRITE_STATUS_CMD_LOWBYTE: u8 = 0x01;
const FLASH_READ_STATUS_CMD_LOWBYTE: u8 = 0x05;
const FLASH_WRITE_ENABLE_CMD: u8 = 0x06;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlashStatusKind {
    Status8Bit,
    Status16BitSingleCommand,
}

#[inline(always)]
fn reg8(addr: usize) -> *mut u8 {
    addr as *mut u8
}

#[inline(always)]
fn mspi_wait() {
    unsafe {
        while (core::ptr::read_volatile(reg8(MSPI_CTRL_ADDR).cast_const()) & FLD_MSPI_BUSY) != 0 {}
    }
}

#[inline(always)]
fn mspi_high() {
    unsafe {
        core::ptr::write_volatile(reg8(MSPI_CTRL_ADDR), FLD_MSPI_CS);
    }
}

#[inline(always)]
fn mspi_low() {
    unsafe {
        core::ptr::write_volatile(reg8(MSPI_CTRL_ADDR), 0);
    }
}

#[inline(always)]
fn mspi_get() -> u8 {
    unsafe { core::ptr::read_volatile(reg8(MSPI_DATA_ADDR).cast_const()) }
}

#[inline(always)]
fn mspi_write(byte: u8) {
    unsafe {
        core::ptr::write_volatile(reg8(MSPI_DATA_ADDR), byte);
    }
}

#[inline(always)]
fn mspi_ctrl_write(byte: u8) {
    unsafe {
        core::ptr::write_volatile(reg8(MSPI_CTRL_ADDR), byte);
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
    // For current HAL use, raw MID is sufficient. Special-casing XTX extended
    // IDs can be added later if needed.
    read_raw_mid()
}

#[unsafe(no_mangle)]
pub extern "C" fn flash_is_zb() -> u8 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn flash_vdd_f_calib() {}
