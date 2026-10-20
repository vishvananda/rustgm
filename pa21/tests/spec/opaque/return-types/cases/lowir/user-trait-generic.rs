trait Echo<T>{fn value(&self)->T;}impl Echo<u8> for u8{fn value(&self)->u8{*self}}fn f<T:Copy+Echo<T>>(x:T)->impl Echo<T>{x}fn main(){let _=f(7u8).value();}
