fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a:'b,'b>(x:&'a u8,y:&'b u8,c:bool)->impl Value{if c{x}else{y}}fn main(){let x=7;let y=11;check(f(&x,&y,true).value()==7);check(f(&x,&y,false).value()==11);}
