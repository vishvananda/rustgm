#![feature(core_intrinsics, discriminant_kind)]
#![allow(internal_features, dead_code)]
use core::intrinsics::discriminant_value;
struct Data { value:u8 }
const UNIT:u8=discriminant_value(&());
const STRUCT:u8=discriminant_value(&Data{value:37});
const SCALAR:u8=discriminant_value(&91u16);
fn main(){println!("{} {} {}",UNIT,STRUCT,SCALAR);}
