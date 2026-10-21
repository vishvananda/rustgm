fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=1u8;let mut f=move|x:&u8|n+*x;let p:&mut dyn FnMut(&u8)->u8=&mut f;check(p(&7)==8);}
