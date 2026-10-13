fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const N:isize=7;enum E{A=N,B}fn f(x:isize)->u8{match x{N=>1,_=>2}}fn main(){check(f(E::A as isize)==1&&f(E::B as isize)==2);}