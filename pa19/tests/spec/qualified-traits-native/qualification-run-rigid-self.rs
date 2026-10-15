fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}fn g<T:A>(x:T)->u8{<T as A>::f(x)}fn main(){let x=g(3u8);check(x==3);}