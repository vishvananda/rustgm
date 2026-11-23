#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::intrinsics::{AtomicOrdering, atomic_fence, atomic_singlethreadfence};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn compiler_boundary(word: *mut u32) -> u32 {
    unsafe {
        *word = 17039;
        atomic_singlethreadfence::<{ AtomicOrdering::SeqCst }>();
        *word
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn thread_boundary(word: *mut u32) -> u32 {
    unsafe {
        *word = 17039;
        atomic_fence::<{ AtomicOrdering::SeqCst }>();
        *word
    }
}

fn experiment() -> i32 {
    let mut word = 0;
    unsafe {
        atomic_singlethreadfence::<{ AtomicOrdering::Acquire }>();
        atomic_singlethreadfence::<{ AtomicOrdering::Release }>();
        atomic_singlethreadfence::<{ AtomicOrdering::AcqRel }>();
        atomic_fence::<{ AtomicOrdering::Acquire }>();
        atomic_fence::<{ AtomicOrdering::Release }>();
        atomic_fence::<{ AtomicOrdering::AcqRel }>();
        if compiler_boundary(&raw mut word) != 17039 || word != 17039 {
            return 1;
        }
        word = 0;
        if thread_boundary(&raw mut word) != 17039 || word != 17039 {
            return 2;
        }
    }
    0
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
