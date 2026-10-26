#![feature(allow_internal_unstable)]
#![allow(dead_code,internal_features)]
#[allow_internal_unstable(final_associated_functions)]macro_rules! m{($i:item)=>{$i}}m!(trait T{final fn value(&self)->u8{7}});
