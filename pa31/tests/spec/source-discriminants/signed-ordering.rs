#![feature(core_intrinsics, discriminant_kind)]
#![allow(internal_features, dead_code)]
use core::intrinsics::discriminant_value;
#[repr(i8)] enum Ordering { Less=-1, Equal=0, Greater=1 }
const LESS:i8=discriminant_value(&Ordering::Less);
const EQUAL:i8=discriminant_value(&Ordering::Equal);
const GREATER:i8=discriminant_value(&Ordering::Greater);
fn main(){println!("{} {} {}",LESS,EQUAL,GREATER);}
