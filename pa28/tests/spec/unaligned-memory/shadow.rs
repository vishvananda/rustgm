mod core{pub mod ptr{pub fn read_unaligned(x:u32)->u32{x}pub fn write_unaligned(x:u32)->u32{x}}}fn main(){let _=core::ptr::read_unaligned(37);let _=core::ptr::write_unaligned(7);}
