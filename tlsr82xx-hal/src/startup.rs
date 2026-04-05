use crate::{clock, interrupt, timer};

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartupState {
    Boot = 0,
    DeepRetention = 1,
    Deep = 2,
}

unsafe extern "C" {
    fn cpu_wakeup_init();
}

#[unsafe(no_mangle)]
pub static mut sysTimerPerUs: u32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn drv_calibration() {}

#[inline(always)]
pub fn init() -> StartupState {
    interrupt::disable();
    interrupt::clear_mask(interrupt::ALL_IRQS);
    interrupt::clear_all_irq_sources();

    unsafe {
        cpu_wakeup_init();
    }
    clock::init(clock::SysClock::Crystal48M);
    unsafe {
        sysTimerPerUs = timer::SYS_TICK_PER_US;
    }

    StartupState::Boot
}
