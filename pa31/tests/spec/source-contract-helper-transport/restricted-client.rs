#![feature(no_core, contracts_internals, contracts, contract_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;
fn condition(_: &'static u8) -> bool { true }
#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 {
    let selected = base::build_check_ensures::<u8, _>(condition);
    if selected(&7) { 7 } else { 3 }
}
