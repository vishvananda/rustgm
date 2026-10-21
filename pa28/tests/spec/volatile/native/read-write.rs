fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x:u32=3;let p=&raw mut x;unsafe{core::ptr::write_volatile(p,17);check(core::ptr::read_volatile(p)==17);}}
