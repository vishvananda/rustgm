#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

const CHECKED: bool = {
    unsafe { core::intrinsics::assume(true) };
    true
};

fn condition(state: &mut u8) -> bool {
    *state = match *state {
        3 => 7,
        _ => 0,
    };
    *state == 7
}

#[unsafe(no_mangle)]
pub unsafe fn preserve_assumption(selected: bool) -> u8 {
    unsafe { core::intrinsics::assume(selected) };
    19
}

fn experiment() -> i32 {
    let mut state = 3;
    let observed = unsafe { preserve_assumption(condition(&mut state)) };
    if CHECKED && observed == 19 && state == 7 {
        0
    } else {
        1
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
