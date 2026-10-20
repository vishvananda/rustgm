trait Q{type Out<T> where for<'a>T:'a;fn f<T:'static>(x:Self::Out<T>);}impl Q for u8{type Out<T>=u8 where for<'a>T:'a;fn f<T:'static>(x:u8){}}
fn main(){}
