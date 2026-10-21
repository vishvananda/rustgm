fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn take<T:Copy>(x:T)->T{let f=||{let g=||x;g()};f()}fn main(){check(take(7u8)==7);}
