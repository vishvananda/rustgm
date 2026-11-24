#![feature(no_core)]
#![no_core]
extern crate base;

#[repr(C)]
struct Packet {
    byte: u8,
    word: u32,
    tail: u16,
}

const BYTES: usize = base::intrinsics::size_of::<Packet>();
const ALIGNMENT: usize = base::intrinsics::align_of::<Packet>();
const DEFAULT_BYTES: usize = <Packet as base::SizedTypeProperties>::SIZE;
const DEFAULT_ALIGNMENT: usize = <Packet as base::SizedTypeProperties>::ALIGN;

#[unsafe(no_mangle)]
pub extern "C" fn direct_size() -> usize { BYTES }

#[unsafe(no_mangle)]
pub extern "C" fn direct_alignment() -> usize { ALIGNMENT }

#[unsafe(no_mangle)]
pub extern "C" fn default_size() -> usize { DEFAULT_BYTES }

#[unsafe(no_mangle)]
pub extern "C" fn default_alignment() -> usize { DEFAULT_ALIGNMENT }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_size() -> usize { <Packet as base::SizedTypeProperties>::SIZE }

#[unsafe(no_mangle)]
pub extern "C" fn runtime_alignment() -> usize { <Packet as base::SizedTypeProperties>::ALIGN }
