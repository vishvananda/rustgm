fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E{A=f()}const fn f()->isize{match 7isize{N=>1,_=>2}}const N:isize=7;fn main(){check(E::A as isize==1);}