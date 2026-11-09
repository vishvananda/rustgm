#![feature(no_core, asm_native_fixture)]
#![no_core]
#![allow(internal_features)]
extern crate base;
// The witness client (amendment 5): its exported C surface is scalar. It
// builds a vector from a local array, streams it through the imported
// `_mm_stream_ps` into a 16-byte-aligned local, and returns 42 only if every
// lane came back. The local is a u128, which x86_64 aligns to 16 bytes and
// which needs no `freeze` lang item to borrow, unlike a local record; the
// foundation declares none. The lanes are read back through a raw pointer as
// f32, the foundation's one Copy scalar.
#[repr(C)]
struct Lanes {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}
#[unsafe(no_mangle)]
pub extern "C" fn rg_stream_check() -> i32 {
    let mut out: u128 = 0;
    let p = &raw mut out as *mut f32;
    unsafe { base::_mm_stream_ps(p, base::rg_vector([1.0, 2.0, 3.0, 4.0])) }
    let lanes = p as *mut Lanes;
    unsafe {
        match (*lanes).a {
            1.0 => match (*lanes).b {
                2.0 => match (*lanes).c {
                    3.0 => match (*lanes).d {
                        4.0 => 42,
                        _ => 4,
                    },
                    _ => 3,
                },
                _ => 2,
            },
            _ => 1,
        }
    }
}
