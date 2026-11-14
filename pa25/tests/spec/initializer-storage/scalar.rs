const fn word(x:u8)->u8{x}static DATA:&u8=&word(7);fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}
fn main(){check(*DATA==7);}
