fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn value<T>(x:T)->T{x}fn main(){let x:u16=value(300);check(x==300);}
