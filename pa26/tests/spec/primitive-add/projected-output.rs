fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn add<T:core::ops::Add<R>,R>(a:T,b:R)->T::Output{a+b}fn main(){check(add(2u16,5u16)==7);}
