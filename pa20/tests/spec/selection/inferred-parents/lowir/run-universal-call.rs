fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W<T>(T);trait Q<'a,P>{type Out;}impl<'a,T:Copy>Q<'a,u16>for W<T>{type Out=T;}fn f(x:&u8)->u8{*x+5}fn main(){let g:for<'a>fn(&'a u8)-><W<_>as Q<'a,_>>::Out=f;check(g(&7)==12);}
