#![allow(static_mut_refs)]static mut X:u8=0;#[deny(static_mut_refs)]fn f()->&'static u8{unsafe{&X}}fn main(){}
