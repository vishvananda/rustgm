fn main(){let mut x=7u32;unsafe{core::ptr::write_unaligned(&raw mut x,37u8);}}
