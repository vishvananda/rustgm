fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make<'a,'b>(x:&'a u8,y:&'b u8,b:bool)->impl A+'b where 'a:'b{W(if b{x}else{y})}fn main(){let n=17u8;let x=make(&n,&n,true);need(&x);check(x.f()==17);}
