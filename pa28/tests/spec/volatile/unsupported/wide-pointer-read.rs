fn main(){let a=[7u8,9];let s:&[u8]=&a;unsafe{let _=core::ptr::read_volatile(&raw const s);}}
