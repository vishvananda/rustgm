struct W<T>(T);trait Q<'a,P>{type Out;}impl<T:Copy>Q<'static,u16>for W<T>{type Out=T;}fn f(_: &u8)->u8{7}fn main(){let _:for<'a>fn(&'a u8)-><W<_>as Q<'a,_>>::Out=f;}
