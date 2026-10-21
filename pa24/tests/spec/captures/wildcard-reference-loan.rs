fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x=7u8;let pair=(&mut x,);x=9;let f=||{let _=pair.0;};f();check(x==9);}
