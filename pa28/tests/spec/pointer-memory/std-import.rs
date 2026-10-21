use std::ptr::{read as take,write as put};fn main(){let mut x=7u32;unsafe{let y=take(&raw const x);put(&raw mut x,y);}}
