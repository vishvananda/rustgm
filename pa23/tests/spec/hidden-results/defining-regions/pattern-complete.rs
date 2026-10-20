fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}const C:(u8,u8)=(1,2);fn make<'a>(x:&'a u8)->impl A+'a{match (1u8,2u8){C=>W(x),_=>W(x)}}fn main(){let n=13u8;let x=make(&n);need(&x);check(x.f()==13);}
