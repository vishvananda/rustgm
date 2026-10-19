struct W<T>(T);trait Mark<P>{}impl Mark<u32>for u8{}trait Q<P,R>{type Out;}impl<T:Mark<R>,R>Q<u16,R>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_,_>>::Out=7u8;}
