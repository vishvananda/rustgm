fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static A:(u8,[u16;2])=(3,[7,11]);const R:&u16=&A.1[1];fn main(){check(*R==11 && A.1[0]==7);}
