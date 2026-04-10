pub fn init() -> tlsr82xx_hal::startup::StartupState {
    let state = tlsr82xx_hal::startup::init();
    tlsr82xx_hal::pm::init(tlsr82xx_hal::pm::Clock32kSource::InternalRc);
    state
}
