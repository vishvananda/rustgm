#![feature(reborrow)]
use std::marker::{Reborrow,CoerceShared};
struct S(u8);impl Reborrow for S{}
