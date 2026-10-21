#[derive(Copy,Clone)]struct S{x:u32}fn main(){let f:unsafe fn(*const S)->S=core::ptr::read_volatile::<S>;let s=S{x:7};unsafe{let _=f(&raw const s);}}
