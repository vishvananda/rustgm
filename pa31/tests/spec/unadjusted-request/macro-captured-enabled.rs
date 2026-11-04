#![allow(internal_features,unused_features,dead_code,unexpected_cfgs)]
#![feature(abi_unadjusted)]
macro_rules! make{($abi:literal)=>{pub extern $abi fn active()->u8{7}}}make!("unadjusted");
