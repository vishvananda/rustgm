fn f<'a>(x:&'a u8)->&'a u8{x}fn main(){let x;{let a=7;x=f(&a);}}
