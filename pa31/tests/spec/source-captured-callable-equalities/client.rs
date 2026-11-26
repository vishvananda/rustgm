#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)] pub extern "C" fn obs_nested() -> u32 {
    let x=41; let mut slot=&x; base::nested(&mut slot)
}
#[unsafe(no_mangle)] pub extern "C" fn obs_referent() -> u32 {
    let x=41; let mut slot=&x; base::nested(&mut slot); *slot
}
#[unsafe(no_mangle)] pub extern "C" fn obs_choose() -> u32 {
    let x=41; let y=73; let mut slot=&x; let (seen,_) = base::choose(&mut slot,&y,false); seen
}
#[unsafe(no_mangle)] pub extern "C" fn obs_changed() -> u32 {
    let x=41; let y=73; let mut slot=&x; base::choose(&mut slot,&y,false); *slot
}
#[unsafe(no_mangle)] pub extern "C" fn obs_kept() -> u32 {
    let x=41; let y=73; let mut slot=&x; base::choose(&mut slot,&y,true); *slot
}
#[unsafe(no_mangle)] pub extern "C" fn obs_free_value() -> u32 {
    let x=29; let (value,_) = base::free_control(&x); value
}
#[unsafe(no_mangle)] pub extern "C" fn obs_free_capture() -> u32 {
    let x=29; let (_,capture) = base::free_control(&x); capture
}
