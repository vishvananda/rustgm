fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W<T>(T);trait Q<P=Self>:Sized{type Out;}impl<T:Copy>Q for W<T>{type Out=T;}fn main(){let p:<W<_>as Q>::Out=7u8;check(p+5==12);}
