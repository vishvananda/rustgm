struct W<T>(T);trait Q<const N:usize>{type Out;}impl<T:Copy>Q<3>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_>>::Out=7u8;}
