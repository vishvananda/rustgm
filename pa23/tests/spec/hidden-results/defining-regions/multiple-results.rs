fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make<'a>(x:&'a u8)->(impl A+'a,impl A+'a){(W(x),W(x))}fn main(){let n=19u8;let (x,y)=make(&n);need(&x);need(&y);check(x.f()+y.f()==38);}
