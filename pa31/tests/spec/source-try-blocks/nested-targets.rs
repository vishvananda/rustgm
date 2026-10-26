#![allow(dead_code,internal_features)]
#![feature(try_blocks)]
fn f()->Result<u8,u8>{let inner:Option<u8>=try{Some(7)?};try{inner.unwrap()}}
