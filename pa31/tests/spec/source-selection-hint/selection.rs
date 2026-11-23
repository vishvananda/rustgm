#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct Witness {
    identifier: u8,
    drops: *mut u8,
}

impl Drop for Witness {
    fn drop(&mut self) {
        unsafe {
            *self.drops = if *self.drops == 0 { 1 } else { 2 };
        }
    }
}

fn selects_once(condition: bool, public: bool) -> bool {
    let mut left_drops = 0_u8;
    let mut right_drops = 0_u8;
    let left_drops = &raw mut left_drops;
    let right_drops = &raw mut right_drops;
    let left = Witness {
        identifier: 3,
        drops: left_drops,
    };
    let right = Witness {
        identifier: 5,
        drops: right_drops,
    };
    let chosen = if public {
        core::hint::select_unpredictable(condition, left, right)
    } else {
        core::intrinsics::select_unpredictable(condition, left, right)
    };
    let expected = if condition { 3 } else { 5 };
    if chosen.identifier != expected {
        return false;
    }
    let unchosen_drops = if public { 1 } else { 0 };
    let (selected, unselected) = if condition {
        (left_drops, right_drops)
    } else {
        (right_drops, left_drops)
    };
    if unsafe { *selected != 0 || *unselected != unchosen_drops } {
        return false;
    }
    core::mem::drop(chosen);
    unsafe { *selected == 1 && *unselected == unchosen_drops }
}

fn experiment() -> i32 {
    if selects_once(false, false)
        && selects_once(true, false)
        && selects_once(false, true)
        && selects_once(true, true)
    {
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
