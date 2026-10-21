fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=5u8;let f=move|x:u8|n+x;let mut r=&f;let p:&mut dyn FnMut(u8)->u8=&mut r;check(p(2)==7);}
