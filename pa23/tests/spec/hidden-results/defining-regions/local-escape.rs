fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}struct W<'a>(&'a u8);impl A for W<'_>{fn f(&self)->u8{*self.0}}fn need<T:Send>(_:&T){}fn make()->impl A{let n=7u8;W(&n)}fn main(){let x=make();need(&x);}
