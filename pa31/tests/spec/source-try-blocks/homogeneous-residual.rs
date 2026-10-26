#![allow(dead_code,internal_features)]
#![feature(try_blocks)]
fn f()->Result<u8,u16>{try{Err::<u8,u8>(3)?}}
