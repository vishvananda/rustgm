trait Q{type Out<T> where for<'a>(u8,T):'static;fn f<T>(x:Self::Out<T>);}impl Q for u8{type Out<T>=u8 where for<'a>(u8,T):'static;fn f<T>(x:u8){}}
fn main(){}
