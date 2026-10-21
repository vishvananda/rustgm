fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn take<T:Copy>(mut x:T,y:T)->T{let f=||x;x=y;f()}fn main(){let _=take(7u8,9);}
