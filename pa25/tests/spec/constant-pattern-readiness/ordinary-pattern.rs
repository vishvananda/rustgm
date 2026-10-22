fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:u8=7;const C:u8=match 7u8{N=>1,_=>2};fn main(){check(C==1);}