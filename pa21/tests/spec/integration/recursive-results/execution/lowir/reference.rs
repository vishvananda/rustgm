fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a>(n:u8,x:&'a u8)->impl Value+'a{if n==0{x}else{f(n-1,x)}}fn main(){let x=113;check(f(4,&x).value()==113);}
