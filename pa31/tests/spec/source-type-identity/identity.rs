#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::any::TypeId;
use core::intrinsics::type_id_eq;

mod left {
    pub struct Marker;
}

mod right {
    pub struct Marker;
}

type Renamed = left::Marker;

const SAME: bool = type_id_eq(TypeId::of::<left::Marker>(), TypeId::of::<Renamed>());
const DIFFERENT: bool = type_id_eq(TypeId::of::<left::Marker>(), TypeId::of::<right::Marker>());

fn compared(left: TypeId, right: TypeId) -> bool {
    left == right
}

fn experiment() -> i32 {
    let original = TypeId::of::<left::Marker>();
    let alias = TypeId::of::<Renamed>();
    let distinct = TypeId::of::<right::Marker>();
    let scalar = TypeId::of::<u8>();
    if SAME
        && !DIFFERENT
        && compared(original, alias)
        && !compared(original, distinct)
        && !compared(scalar, TypeId::of::<i8>())
        && !compared(TypeId::of::<[u8; 2]>(), TypeId::of::<[u8; 3]>())
        && !compared(TypeId::of::<fn(&u8)>(), TypeId::of::<fn(&'static u8)>())
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
