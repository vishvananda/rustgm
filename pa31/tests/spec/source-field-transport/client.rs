#![feature(no_core, field_projections, field_representing_type_raw, reborrow_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

#[repr(C)]
struct Record { first: u32, second: u8 }
type Selected = base::field::FieldRepresentingType<Record, 0, 1>;

#[unsafe(no_mangle)]
pub extern "C" fn observed() -> usize {
    <Selected as base::field::Field>::OFFSET
}
