#![feature(allow_internal_unstable)]
#[allow_internal_unstable(optimize_attribute)]macro_rules! make{($m:meta)=>{#[$m]pub fn f()->u32{7}}}make!(optimize(size));
