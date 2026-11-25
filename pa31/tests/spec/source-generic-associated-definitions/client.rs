#![feature(no_core)]
#![no_core]

extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed_cast_lane() -> u32 {
    let lane: <base::Lane<u32, 4> as base::Castable>::Cast<u16> = base::Lane { head: 29 };
    <base::Lane<u32, 4> as base::Castable>::cast_tag::<u16>(lane)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_mixed_view() -> u32 {
    let elem: u8 = 11;
    let view: <base::Lane<u32, 2> as base::Castable>::View<'_, u8> = (&elem, 30);
    base::read_view::<base::Lane<u32, 2>, u8>(view)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_ptr_lane() -> u32 {
    let value: u32 = 77;
    let lane: <base::PtrLane<*const u8, 2> as base::CastablePtr>::CastPtr<u32> =
        base::PtrLane { raw: &value };
    unsafe { *lane.raw }
}
