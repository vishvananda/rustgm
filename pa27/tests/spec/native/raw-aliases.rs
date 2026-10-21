fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x=3u8;let p=&raw mut x;let q=p;unsafe{*p=9;check(*q==9);}}