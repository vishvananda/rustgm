fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f=|x|x;let x:u8=f(7);check(x==7);}
