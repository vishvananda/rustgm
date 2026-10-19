struct W<T>(T);trait Q<P>{type Out;}impl<T:Copy>Q<u16>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_>>::Out=7u8;}
