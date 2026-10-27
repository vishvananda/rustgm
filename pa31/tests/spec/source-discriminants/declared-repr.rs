#![feature(core_intrinsics, discriminant_kind)]
#![allow(internal_features, dead_code)]
use core::intrinsics::discriminant_value;
enum DefaultTag { A=-7, B=19 }
#[repr(C)] enum CTag { A=-3, B=27 }
#[repr(u16)] enum Payload { A(u8)=300, B{v:u8}=900 }
const DEFAULT:isize=discriminant_value(&DefaultTag::A);
const C:isize=discriminant_value(&CTag::B);
const DATA:u16=discriminant_value(&Payload::A(5));
fn main(){println!("{} {} {}",DEFAULT,C,DATA);}
