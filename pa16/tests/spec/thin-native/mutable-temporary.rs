fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let r=&mut 40u64;*r+=2;check(*r==42);}
