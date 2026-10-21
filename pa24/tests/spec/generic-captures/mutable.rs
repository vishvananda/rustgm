fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn replace<T>(x:T,y:T)->T{let mut v=x;let mut f=|n:T|{v=n;};f(y);v}fn main(){check(replace(3u8,7)==7);check(replace(2u16,900)==900);}
