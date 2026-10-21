fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x:u8=7;let y:u8=11;let mut p=&raw const x;unsafe{core::ptr::write_volatile(&raw mut p,&raw const y);let q=core::ptr::read_volatile(&raw const p);check(*q==11);}}
