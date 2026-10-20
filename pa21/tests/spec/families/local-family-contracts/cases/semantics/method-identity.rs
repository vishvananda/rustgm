trait Q{type Out<T>;fn f<T>(x:T)->Self::Out<T>;}impl Q for u8{type Out<T>=T;fn f<T>(x:T)->T{x}}
fn main(){}
