fn f<'a>(_:&'a u8)->impl Copy+'static{()}fn main(){let x;{let a=7;x=f(&a);}let _=x;}
