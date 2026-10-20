fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a,'b>(x:&'a u8,_:&'b u8)->impl Value+'a{x}fn main(){let a=19;let x;{let b=1;x=f(&a,&b);}check(x.value()==19);}
