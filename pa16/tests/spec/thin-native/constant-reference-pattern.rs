fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const C:&u8=&7;fn f(x:&u8)->u8{match x{C=>1,_=>2}}fn main(){check(f(&7)==1 && f(&9)==2);}
