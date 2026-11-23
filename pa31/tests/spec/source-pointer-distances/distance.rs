#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

fn run() -> i32 {
    let values = [19u32, 42, 73];
    let base = (&raw const values) as *const u32;
    unsafe {
        let middle = core::intrinsics::offset(base, 1usize);
        let last = core::intrinsics::offset(base, 2isize);
        if core::intrinsics::ptr_offset_from(last, base) != 2
            || core::intrinsics::ptr_offset_from(base, last) != -2
            || core::intrinsics::ptr_offset_from(middle, middle) != 0
        {
            return 1;
        }
        if core::intrinsics::ptr_offset_from_unsigned(last, base) != 2
            || core::intrinsics::ptr_offset_from_unsigned(middle, middle) != 0
        {
            return 2;
        }
    }
    0
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! { loop {} }

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 { run() }

#[cfg(not(guest_probe))]
fn main() { assert_eq!(run(), 0); }
