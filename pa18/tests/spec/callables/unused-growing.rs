fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<T>(){f::<(T,T)>();}fn main(){let _=f::<u8>;check(true);}
