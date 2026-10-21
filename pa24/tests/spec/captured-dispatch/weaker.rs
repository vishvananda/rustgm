fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=7u8;let mut f=move|n:u8|x+n;let p:&mut dyn FnMut(u8)->u8=&mut f;check(p(2)==9);}
