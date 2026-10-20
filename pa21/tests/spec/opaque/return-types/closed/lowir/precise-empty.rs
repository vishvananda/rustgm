fn make<'a>(x:&'a u8)->impl Copy+use<>{7u8}fn take(x:impl Copy){}fn main(){let x;{let v=7;x=make(&v);}take(x);}
