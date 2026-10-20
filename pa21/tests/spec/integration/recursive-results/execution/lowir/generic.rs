fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<T:Copy+Value>(n:u8,x:T)->impl Value{if n==0{x}else{f(n-1,x)}}fn main(){check(f(4,109u8).value()==109);}
