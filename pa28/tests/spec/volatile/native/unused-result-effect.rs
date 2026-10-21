fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x:u8=0;unsafe{core::ptr::write_volatile(&raw mut x,29);let _=core::ptr::read_volatile(&raw const x);}check(x==29);}
