fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(Copy,Clone)]struct S(u8);trait A{type Out:Copy;}impl A for u8{type Out=S;}fn f<T:A>(x:T::Out)->(T::Out,T::Out){(x,x)}fn main(){let p=f::<u8>(S(7));check(p.0.0==7&&p.1.0==7)}
