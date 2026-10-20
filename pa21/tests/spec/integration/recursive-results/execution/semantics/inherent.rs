fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}struct S;impl S{fn f(n:u8)->impl Value{if n==0{139u8}else{Self::f(n-1)}}}fn main(){check(S::f(4).value()==139);}
