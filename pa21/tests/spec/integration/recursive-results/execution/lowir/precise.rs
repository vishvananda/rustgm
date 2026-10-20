fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a>(n:u8,x:&'a u8)->impl Value+use<>{if n==0{137u8}else{let y=2;f(n-1,&y)}}fn main(){check(f(4,&1).value()==137);}
