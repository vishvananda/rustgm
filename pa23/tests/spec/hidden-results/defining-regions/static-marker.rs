fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}unsafe impl<'a:'static> Send for W<'a>{}fn make<'a>(x:&'a u8)->impl A+'a{W(x)}static N:u8=7;fn main(){let x=make(&N);need(&x);check(x.f()==7);}
