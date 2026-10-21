#![allow(static_mut_refs)]static mut X:u8=0;fn f(){unsafe{#[deny(static_mut_refs)]let _=&X;}}fn main(){}
