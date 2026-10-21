fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=1u8;let mut f=move|x:&u8|n+*x;let p:&mut dyn Fn(&u8)->u8=&mut f;let q:&mut dyn FnMut(&u8)->u8=p;check(q(&7)==8);}
