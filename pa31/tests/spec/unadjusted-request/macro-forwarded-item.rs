#![allow(internal_features,dead_code)]
#![feature(allow_internal_unstable)]
#[allow_internal_unstable(abi_unadjusted)]macro_rules!make{($item:item)=>{$item}}make!(pub extern "unadjusted" fn active()->u8{7});
