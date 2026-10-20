struct W<T>(T);impl<T>Drop for W<T>{fn drop(&mut self){}}fn f<'a>(x:&'a u8)->impl Copy{x}fn main(){let x;{let a=1;x=W(f(&a));}let _=x;}
