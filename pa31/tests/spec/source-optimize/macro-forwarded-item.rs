#![feature(allow_internal_unstable)]
#[allow_internal_unstable(optimize_attribute)]macro_rules! make{($i:item)=>{$i}}make!(#[optimize(size)]pub fn f()->u32{7});
