#[derive(Copy,Clone)]#[repr(align(8))]enum E{Only(u8)}const N:u8=unsafe{let e=E::Only(37);core::ptr::read(((&raw const e)as *const u8).add(1))};fn main(){let _=N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
