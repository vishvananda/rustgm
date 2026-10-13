fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
enum E{A,B,C}const N:isize=E::C as isize;fn f(x:isize)->u8{match x{N=>1,_=>2}}fn main(){check(f(2)==1&&f(1)==2);}