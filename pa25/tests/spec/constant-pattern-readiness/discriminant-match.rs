fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:isize=7;enum E{A=match 7isize{N=>1,_=>2}}fn main(){check(E::A as isize==1);}