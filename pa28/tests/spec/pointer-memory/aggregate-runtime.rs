#[derive(Copy,Clone)]#[repr(C)]struct P{a:u8,b:u32}fn main(){let mut p=P{a:3,b:7};unsafe{let q=core::ptr::read(&raw const p);core::ptr::write(&raw mut p,q);}}
