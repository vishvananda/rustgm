trait Q{type Out<T:Copy>;fn f<T:Copy>(x:T)->Self::Out<T>;}impl Q for u8{type Out<T:Copy>=u8;fn f<T:Copy>(x:T)->u8{0}}
fn main(){}
