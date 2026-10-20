fn f<'a>(x:&'a u8)->impl Copy{x}fn main(){let x;{let a=7;x=f(&a);}let _=x;}
