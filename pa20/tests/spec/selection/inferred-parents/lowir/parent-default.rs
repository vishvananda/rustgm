struct W<T>(T);trait Q<P=u16>{type Out;}impl<T:Copy>Q for W<T>{type Out=T;}fn main(){let _: <W<_>as Q>::Out=7u8;}
