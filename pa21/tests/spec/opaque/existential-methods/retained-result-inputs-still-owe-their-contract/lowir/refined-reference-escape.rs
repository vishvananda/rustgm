trait Make{fn make<'a>(x:&'a u8)->impl Copy+'a;}impl Make for u8{fn make<'a>(x:&'a u8)->&'a u8{x}}fn main(){let x;{let y=1;x=<u8 as Make>::make(&y);}let _=x;}
