#![feature(core_intrinsics, discriminant_kind)]
#![allow(internal_features, dead_code)]
use core::intrinsics::discriminant_value;
struct Data;
impl core::marker::DiscriminantKind for Data {type Discriminant=u8;}
fn main(){}
