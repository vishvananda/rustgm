fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}unsafe impl<'a:'static> Send for W<'a>{}fn need<T:Send>(_:&T){}fn make<'a,'b>(x:&'a u8,y:&'b u8)->(impl A+use<'a>,impl A+use<'b>){(W(x),W(y))}static N:u8=19;fn main(){let m=23u8;let (x,y)=make(&N,&m);need(&y);need(&x);}
