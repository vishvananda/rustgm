fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){let mut x=7u8;let r=&mut x;let f=||->&u8{r};check(*f()==7);}
