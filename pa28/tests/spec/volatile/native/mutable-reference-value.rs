fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x:u8=7;let mut y:u8=11;let mut r=&mut x;unsafe{core::ptr::write_volatile(&raw mut r,&mut y);let p=core::ptr::read_volatile(&raw const r);*p=17;}check(y==17);}
