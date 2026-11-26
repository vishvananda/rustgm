#![feature(no_core,f16)]
#![no_core]
extern crate base;
unsafe extern "C" { fn seed_half() -> u16; }
#[unsafe(no_mangle)]
pub extern "C" fn runtime_cast() -> u32 {
    unsafe {
        let value = base::transmute_unchecked::<u16,f16>(seed_half());
        base::transmute_unchecked::<f32,u32>(base::half_to_single(value))
    }
}
