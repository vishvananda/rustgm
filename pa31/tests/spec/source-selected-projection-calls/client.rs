#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)] pub extern "C" fn obs_direct() -> u32 { base::direct(11) }
#[unsafe(no_mangle)] pub extern "C" fn obs_other() -> u32 { base::other(22) }
#[unsafe(no_mangle)] pub extern "C" fn obs_borrowed() -> u32 { *base::borrowed(&base::Cell(33)) }
#[unsafe(no_mangle)] pub extern "C" fn obs_generic() -> u32 { base::generic(&base::Cell(44)) }
#[unsafe(no_mangle)] pub extern "C" fn obs_forwarded() -> u32 { base::forwarded(55) }
#[unsafe(no_mangle)] pub extern "C" fn obs_index_left() -> u32 { base::indexed(&base::Pair(66,77),false) }
#[unsafe(no_mangle)] pub extern "C" fn obs_index_right() -> u32 { base::indexed(&base::Pair(66,77),true) }
#[unsafe(no_mangle)] pub extern "C" fn obs_index_mut() -> u32 { base::indexed_mut(&mut base::Pair(1,2),88) }
#[unsafe(no_mangle)] pub extern "C" fn obs_implicit() -> u32 {
    base::implicit(&base::Wrapper(false, base::Pair(99,100)))
}
