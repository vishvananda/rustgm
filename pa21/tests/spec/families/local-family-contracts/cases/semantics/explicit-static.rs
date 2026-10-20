trait Q{type Out<T:'static>;fn f<T>(x:Self::Out<T>);}impl Q for u8{type Out<T>=u8 where T:'static;fn f<T>(x:u8){}}
fn main(){}
