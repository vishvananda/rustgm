#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::intrinsics::{
    AtomicOrdering, atomic_cxchg, atomic_cxchgweak, atomic_load, atomic_max, atomic_nand,
    atomic_umin, atomic_xadd, atomic_xchg, atomic_xsub,
};

#[repr(C)]
struct Bytes {
    first: u8,
    second: u8,
}

fn experiment() -> i32 {
    let mut counter = 13_u32;
    let mut word = -7_i16;
    let mut bits = 0b0000_1010_u8;
    let mut bytes = Bytes {
        first: 11,
        second: 19,
    };
    let first = &raw mut bytes as *mut u8;
    let second = &raw mut bytes.second;
    let mut pointer = first;
    unsafe {
        if atomic_xadd::<_, _, { AtomicOrdering::Relaxed }>(&raw mut counter, 7_u32) != 13
            || counter != 20
        {
            return 9;
        }
        if atomic_xchg::<_, { AtomicOrdering::Acquire }>(&raw mut word, 11) != -7 {
            return 1;
        }
        if atomic_xsub::<_, _, { AtomicOrdering::Release }>(&raw mut word, 13_i16) != 11 {
            return 2;
        }
        let (observed, exchanged) = atomic_cxchg::<
            _,
            { AtomicOrdering::AcqRel },
            { AtomicOrdering::Acquire },
        >(&raw mut word, -2, -9);
        if observed != -2 || !exchanged {
            return 3;
        }
        let (observed, exchanged) = atomic_cxchgweak::<
            _,
            { AtomicOrdering::Relaxed },
            { AtomicOrdering::SeqCst },
        >(&raw mut word, -2, 20);
        if observed != -9 || exchanged {
            return 4;
        }
        if atomic_max::<_, { AtomicOrdering::Acquire }>(&raw mut word, -4) != -9 || word != -4 {
            return 5;
        }
        if atomic_nand::<_, _, { AtomicOrdering::Release }>(&raw mut bits, 0b0000_1100_u8)
            != 0b0000_1010
            || bits != 0b1111_0111
        {
            return 6;
        }
        if atomic_umin::<_, { AtomicOrdering::Relaxed }>(&raw mut bits, 5) != 0b1111_0111
            || bits != 5
        {
            return 7;
        }
        let previous = atomic_xadd::<_, _, { AtomicOrdering::SeqCst }>(&raw mut pointer, 1_usize);
        let current = atomic_load::<_, { AtomicOrdering::Relaxed }>(&raw const pointer);
        if previous == first && current == second && *current == 19 {
            0
        } else {
            8
        }
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
