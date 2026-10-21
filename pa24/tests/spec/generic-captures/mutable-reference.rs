fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add<T:Copy>(x:&mut T,y:T){let mut f=||{*x=y;};f();}fn main(){let mut a=7u8;add(&mut a,9u8);check(a==9);}
