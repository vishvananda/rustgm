#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

struct Item(u8);
impl base::Marker for Item {
    fn id(&self) -> u8 { self.0 }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_void() -> u8 {
    match base::ok_value(41) {
        base::Outcome::Ok(v) => v,
        base::Outcome::Err(never) => match never {},
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_dyn_inherent() -> u8 {
    let item = Item(7);
    let m: &dyn base::Marker = &item;
    m.tag()
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_dyn_send_inherent() -> u8 {
    let item = Item(0);
    let m: &(dyn base::Marker + base::Send) = &item;
    m.shared_tag()
}
