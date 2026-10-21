fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut b=7u8;let x=&mut b;let mut f=||{let _=&x;*x+=2;};let y=&x;f();let _=**y;}
