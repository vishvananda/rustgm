fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f(n:u8)->(impl Value,impl Value){if n==0{(127u8,131u8)}else{f(n-1)}}fn main(){let p=f(4);check(p.0.value()==127);check(p.1.value()==131);}
