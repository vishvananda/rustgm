const fn word(x:u8)->u8{x}static DATA:&u8=&{let mut x=word(7);x=9;x};
fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}
fn main(){check(*DATA==9);}
