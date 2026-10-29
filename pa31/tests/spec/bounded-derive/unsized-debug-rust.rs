#![allow(dead_code)]
#[derive(Debug)]struct Tail<T:?Sized>{value:T}
fn debug_tail(value:&Tail<[u8]>) {let _: &dyn core::fmt::Debug=&value;}
