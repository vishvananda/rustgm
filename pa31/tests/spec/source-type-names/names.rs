#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::{any::type_name, mem::MaybeUninit};

mod model {
    pub struct Holder<'a, const N: usize>(pub &'a [u16; N]);
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

fn check() -> i32 {
    if type_name::<Option<model::Holder<'static, 3>>>()
        != "core::option::Option<app::model::Holder<'_, 3>>"
    {
        return 1;
    }
    if type_name::<MaybeUninit<model::Holder<'static, 3>>>()
        != "core::mem::maybe_uninit::MaybeUninit<app::model::Holder<'_, 3>>"
    {
        return 2;
    }
    if type_name::<&'static mut [&'static u16]>() != "&mut [&u16]" {
        return 3;
    }
    0
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    check()
}

#[cfg(not(guest_probe))]
fn main() {
    std::process::exit(check());
}
