struct W<T>(T);trait Q<'a,P>{type Out;}impl<'a,T:Copy>Q<'a,u16>for W<T>{type Out=T;}type F<T,P>=for<'a>fn(&'a u8)-><W<T>as Q<'a,P>>::Out;fn f(_: &u8)->u8{7}fn main(){let _:F<_,_>=f;}
