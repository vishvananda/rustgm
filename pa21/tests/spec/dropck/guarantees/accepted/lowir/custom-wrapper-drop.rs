fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct W<T>(T);impl<T>Drop for W<T>{fn drop(&mut self){check(true);}}fn f<'a>(_:&'a u8)->impl Copy+'static{7u8}fn main(){let x;{let a=1;x=W(f(&a));}let _=x;}
