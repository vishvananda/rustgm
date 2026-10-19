fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W<T>(T);trait Q<P>{type Out;}impl<T:Copy>Q<u16>for W<T>{type Out=(T,T);}fn main(){let p:<W<_>as Q<_>>::Out=(7u8,9u8);check(p.0+p.1==16);}
