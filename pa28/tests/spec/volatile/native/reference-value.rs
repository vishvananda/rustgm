fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x:u8=7;let r=&x;unsafe{let s=core::ptr::read_volatile(&raw const r);check(*s==7);}}
