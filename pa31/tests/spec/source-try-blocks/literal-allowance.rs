#![allow(dead_code,internal_features)]
#![feature(allow_internal_unstable)]
#[allow_internal_unstable(try_blocks)]macro_rules! m{()=>{try{7}}}
fn f()->Option<u8>{m!()}
