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

fn rejected<const VALUE: Tag>() -> u8 {
    base::marker()
}
