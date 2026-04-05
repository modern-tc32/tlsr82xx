pub fn drv_platform_init() -> i32 {
    let _ = tlsr82xx_hal::startup::init();
    0
}
