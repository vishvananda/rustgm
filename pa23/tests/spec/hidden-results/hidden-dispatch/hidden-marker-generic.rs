fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn make()->impl A{9u8}fn pass<T:Send>(x:T)->T{x}fn main(){let x=pass(make());check(x.f()==9);}
