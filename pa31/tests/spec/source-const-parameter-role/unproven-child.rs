#![feature(no_core, min_adt_const_params)]
#![no_core]
#![allow(internal_features, incomplete_features)]
extern crate base;

struct Bad {
    item: f32,
}
impl base::Structural for Bad {}
impl base::Equality for Bad {}
impl base::Key for Bad {}
