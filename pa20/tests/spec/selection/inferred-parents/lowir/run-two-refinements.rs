fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W<T>(T);trait Mark<P>{}impl Mark<u32>for u8{}trait Q<P,R>{type Out;}impl<T:Mark<R>,R>Q<u16,R>for W<T>{type Out=T;}fn main(){let p:<W<_>as Q<_,_>>::Out=7u8;check(p*6==42);}
