extern crate base;

use base::Carrier as Renamed;

const _: [u8; 7] = [0; base::FREE[0] as usize];
const _: [u8; 13] = [0; Renamed::ASSOCIATED[0] as usize];

#[unsafe(no_mangle)]
pub extern "C" fn read_free() -> u8 {
    base::FREE[1]
}

#[unsafe(no_mangle)]
pub extern "C" fn read_associated() -> u8 {
    Renamed::ASSOCIATED[1]
}
