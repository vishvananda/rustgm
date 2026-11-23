#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

extern crate alloc;

use alloc::vec;
use core::cell::Cell;

#[cfg(guest_probe)]
mod host {
    use core::{alloc::{GlobalAlloc, Layout}, ptr};

    #[link(name = "c")]
    unsafe extern "C" {
        fn posix_memalign(out: *mut *mut u8, align: usize, bytes: usize) -> i32;
        fn free(pointer: *mut u8);
        pub fn _Exit(status: i32) -> !;
    }

    struct Allocator;

    // POSIX allocates at the requested alignment and accepts free.
    unsafe impl GlobalAlloc for Allocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let mut allocation = ptr::null_mut();
            let align = if layout.align() < 8 { 8 } else { layout.align() };
            if unsafe { posix_memalign(&mut allocation, align, layout.size()) } == 0 {
                allocation
            } else {
                ptr::null_mut()
            }
        }

        unsafe fn dealloc(&self, pointer: *mut u8, _: Layout) {
            unsafe { free(pointer) };
        }
    }

    #[global_allocator]
    static ALLOCATOR: Allocator = Allocator;
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    unsafe { host::_Exit(76) }
}

struct Owned<'a>(u16, &'a Cell<u16>);

impl Drop for Owned<'_> {
    fn drop(&mut self) {
        self.1.set(self.1.get().wrapping_add(self.0));
    }
}

fn check() -> i32 {
    let values = vec![3_u16, 251, 517];
    if !matches!(values.as_slice(), [3, 251, 517]) {
        return 1;
    }

    let tally = Cell::new(0);
    let owned = vec![Owned(7, &tally), Owned(13, &tally), Owned(31, &tally)];
    if !matches!(owned.as_slice(), [Owned(7, _), Owned(13, _), Owned(31, _)]) {
        return 2;
    }
    if tally.get() != 0 {
        return 3;
    }
    drop(owned);
    if tally.get() != 51 {
        return 4;
    }

    let empty_values = vec![(), (), ()];
    if !matches!(empty_values.as_slice(), [(), (), ()]) {
        return 5;
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
