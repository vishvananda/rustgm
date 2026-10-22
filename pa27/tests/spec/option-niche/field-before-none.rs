static S:u32=37;const fn run()->u32{let mut v=Some(&S);let p=match &mut v{Some(r)=>r as *mut &u32,None=>return 0};v=None;unsafe{*p=&S;}match v{Some(r)=>*r,None=>0}}const N:u32=run();fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
