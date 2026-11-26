extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn first(value: u8) -> u8 {
    base::Reading(value).first()
}

#[unsafe(no_mangle)]
pub extern "C" fn second(value: u8) -> u8 {
    base::Reading(value).second()
}

#[unsafe(no_mangle)]
pub extern "C" fn values() -> u8 {
    base::NAMED ^ base::STORED
}
