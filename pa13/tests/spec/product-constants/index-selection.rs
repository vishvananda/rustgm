fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const C:[u8;2]=[2,7];const D:u8=C[1];fn f(x:u8)->u8{match x{D=>1,_=>2}}fn main(){check(f(7)==1&&f(2)==2);}