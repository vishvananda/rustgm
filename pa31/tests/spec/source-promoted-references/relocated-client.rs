#![feature(no_core)]
#![no_core]
extern crate base;

static SNAPSHOT: *mut base::Ops = base::STORED.slot.item;
static RAW_SNAPSHOT: *mut base::Ops = unsafe { *(&raw const base::STORED.slot.item) };
const MARKER: u8 = base::MARKER;
const ADDRESS: *const base::Holder<*mut base::Ops> = &raw const base::STORED;
const SHARED: &'static base::Holder<*mut base::Ops> = &base::STORED;

#[unsafe(no_mangle)]
pub extern "C" fn observe() -> i32 {
    let raw = unsafe { &*(*ADDRESS).slot.item };
    let shared = unsafe { &*SHARED.slot.item };
    let raw_snapshot = unsafe { &*RAW_SNAPSHOT };
    let snapshot = unsafe { &*SNAPSHOT };
    let mut text = base::Arguments { text: "client" };
    if (raw.retry)(MARKER) || (shared.retry)(MARKER) || (raw_snapshot.retry)(MARKER) {
        1
    } else {
        match (snapshot.message)(11, &mut text) {
            base::Opt::Some(value) => value as i32,
            base::Opt::None => 3,
        }
    }
}
