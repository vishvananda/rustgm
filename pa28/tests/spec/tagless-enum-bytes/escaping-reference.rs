enum E<T>{Only(T)}const N:E<&u32>=unsafe{let v=37;let e=E::Only(&v);let p=&raw const e;core::ptr::read(p)};fn main(){let _=&N;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
