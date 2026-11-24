#![feature(no_core, min_adt_const_params)]
#![no_core]
#![allow(internal_features, incomplete_features)]
extern crate base;

enum Tag {
    First,
    Second,
}
impl base::Structural for Tag {}
impl base::Equality for Tag {}
impl base::Key for Tag {}

fn selected<const VALUE: Tag>() -> u8 {
    match VALUE {
        Tag::First => base::marker(),
        Tag::Second => 19,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn first() -> u8 {
    selected::<{ Tag::First }>()
}

#[unsafe(no_mangle)]
pub extern "C" fn second() -> u8 {
    selected::<{ Tag::Second }>()
}
