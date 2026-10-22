fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const M:isize=7;const N:isize=match 7isize{M=>1,_=>2};enum E{A=N}fn main(){check(E::A as isize==1);}