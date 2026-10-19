fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W<T>(T);trait Q<'a,'b,P>{type Out;}impl<'a,'b,T:Copy>Q<'a,'b,u16>for W<T>{type Out=T;}static X:u8=7;fn add(x:&u8,y:&u8)->u8{*x+*y}fn f<'a>(g:for<'b>fn(&'a u8,&'b u8)->u8){check(g(&X,&9)==16);}fn main(){let g:for<'a>fn(for<'b>fn(&'a u8,&'b u8)-><W<_>as Q<'a,'b,_>>::Out)=f;g(add);}
