#![cfg_attr(guest_probe, feature(core_intrinsics))]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[cfg(guest_probe)]
use core::intrinsics::slice_get_unchecked;

#[repr(C)]
struct Entry {
    a: u8,
    b: u8,
    c: u8,
}

#[cfg(guest_probe)]
fn experiment() -> i32 {
    let mut values = [
        Entry { a: 7, b: 11, c: 13 },
        Entry {
            a: 17,
            b: 19,
            c: 23,
        },
    ];
    let wide: *mut [Entry] = &raw mut values;
    let placed: *mut Entry = unsafe { slice_get_unchecked(wide, 1) };
    unsafe { (*placed).b = 29 };
    let unchanged: *const Entry = unsafe { slice_get_unchecked(wide as *const [Entry], 0) };
    if unsafe { (*unchanged).b } != 11 {
        return 1;
    }
    let shared: &[Entry] = &values;
    let read: &Entry = unsafe { slice_get_unchecked(shared, 1) };
    if read.a != 17 || read.b != 29 || read.c != 23 {
        return 2;
    }
    let mutable: &mut [Entry] = &mut values;
    let edit: &mut Entry = unsafe { slice_get_unchecked(mutable, 0) };
    edit.c = 37;
    let shared: &[Entry] = &values;
    let read: &Entry = unsafe { slice_get_unchecked(shared, 0) };
    if read.c != 37 {
        return 3;
    }
    let units = [(); 2];
    let wide: *const [()] = &raw const units;
    let item: *const () = unsafe { slice_get_unchecked(wide, 1) };
    if item == (&raw const units) as *const () {
        0
    } else {
        4
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

#[cfg(not(guest_probe))]
fn experiment() -> i32 {
    let mut values = [
        Entry { a: 7, b: 11, c: 13 },
        Entry {
            a: 17,
            b: 19,
            c: 23,
        },
    ];
    let mutable: &mut [Entry] = &mut values;
    unsafe { mutable.get_unchecked_mut(1) }.b = 29;
    let shared: &[Entry] = &values;
    if unsafe { shared.get_unchecked(0) }.b != 11 {
        return 1;
    }
    let last = unsafe { shared.get_unchecked(1) };
    if last.a != 17 || last.b != 29 || last.c != 23 {
        return 2;
    }
    let mutable: &mut [Entry] = &mut values;
    unsafe { mutable.get_unchecked_mut(0) }.c = 37;
    let shared: &[Entry] = &values;
    if unsafe { shared.get_unchecked(0) }.c != 37 {
        return 4;
    }
    let units = [(); 2];
    let slice: &[()] = &units;
    let item = unsafe { slice.get_unchecked(1) };
    if &raw const *item == (&raw const units) as *const () {
        0
    } else {
        3
    }
}
