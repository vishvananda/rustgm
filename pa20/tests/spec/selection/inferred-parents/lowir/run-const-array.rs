fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W<T,const N:usize>(T);trait Q<P>{type Out;}impl<T:Copy,const N:usize>Q<u16>for W<T,N>{type Out=[T;N];}fn main(){let p:<W<_,_>as Q<_>>::Out=[7u8,9u8,11u8];check(p[0]+p[2]==18);}
