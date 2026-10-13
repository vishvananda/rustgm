fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static N:(u8,u8)=(7,9);const C:&u8=&N.1;fn f(x:&u8)->u8{match x{C=>1,_=>2}}fn main(){check(f(&9)==1 && f(&7)==2);}
