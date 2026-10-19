struct W<T>(T);trait Mark{}impl Mark for u8{}trait Q<P>{type Out;}impl<T:Mark>Q<u16>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_>>::Out=7u8;}
