fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=[3u32,5];let r=&a;let f=||r[1];check(f()==5);}
