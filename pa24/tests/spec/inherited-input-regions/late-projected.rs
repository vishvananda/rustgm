fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Element{type Out:Copy;}impl Element for u8{type Out=u8;}fn copy<T:Copy>(pair:(&T,&u8))->T{*pair.0}fn f<'a,T:Element>(x:&'a T::Out)->T::Out{let c=|pair:(&T::Out,_)|copy(pair);c((x,&7u8))}fn main(){check(f::<u8>(&7u8)==7);}
