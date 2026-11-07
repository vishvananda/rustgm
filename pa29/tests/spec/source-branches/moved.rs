#![feature(no_core)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { let packet = base::Packet { value: 1i32 }; let kept = base::swap(true, packet, base::Packet { value: 2i32 }); packet.value }
