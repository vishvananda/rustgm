#![feature(optimize_attribute)]
#[optimize(size)]pub fn f()->u32{let x=1;let y=&x;x=2;*y}
