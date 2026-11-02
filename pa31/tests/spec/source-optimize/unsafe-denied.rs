#![feature(optimize_attribute)]
#[optimize(none)]unsafe fn f()->u32{7}pub fn g()->u32{f()}
