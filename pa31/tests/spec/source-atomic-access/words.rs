#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::intrinsics::{AtomicOrdering, atomic_load, atomic_store};

fn experiment() -> i32 {
    let mut byte = 0_u8;
    let mut half = 0_u16;
    let mut wide = 0_i64;
    let mut payload = 23_u32;
    let address = &raw mut payload;
    let mut pointer = core::ptr::null_mut();
    unsafe {
        atomic_store::<_, { AtomicOrdering::Relaxed }>(&raw mut byte, 197);
        if atomic_load::<_, { AtomicOrdering::Relaxed }>(&raw const byte) != 197 {
            return 1;
        }
        atomic_store::<_, { AtomicOrdering::Release }>(&raw mut half, 0x8675);
        if atomic_load::<_, { AtomicOrdering::Acquire }>(&raw const half) != 0x8675 {
            return 2;
        }
        atomic_store::<_, { AtomicOrdering::SeqCst }>(&raw mut wide, -0x0154_7654_3234);
        if atomic_load::<_, { AtomicOrdering::SeqCst }>(&raw const wide) != -0x0154_7654_3234 {
            return 3;
        }
        atomic_store::<_, { AtomicOrdering::Release }>(&raw mut pointer, address);
        let read = atomic_load::<_, { AtomicOrdering::Acquire }>(&raw const pointer);
        if read == address && *read == 23 { 0 } else { 4 }
    }
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    experiment()
}

#[cfg(not(guest_probe))]
fn main() {
    assert_eq!(experiment(), 0);
}
