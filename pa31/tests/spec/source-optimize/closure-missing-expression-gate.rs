#![feature(optimize_attribute)]
pub fn f()->u32{let c=#[optimize(size)]||7;c()}
