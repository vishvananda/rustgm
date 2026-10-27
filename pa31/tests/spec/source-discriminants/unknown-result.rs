#![feature(core_intrinsics, discriminant_kind)]
#![allow(internal_features, dead_code)]
use core::intrinsics::discriminant_value;
fn unknown<T>(value:&T)->u8 {discriminant_value(value)}
fn main(){}
