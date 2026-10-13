fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const C:(u8,bool)=(7,true);fn f(x:&(u8,bool))->u8{match x{&C=>1,_=>2}}fn main(){check(f(&(7,true))==1 && f(&(9,true))==2);}
