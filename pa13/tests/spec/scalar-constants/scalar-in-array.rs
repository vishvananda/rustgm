fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
const N:u8=7;fn f(x:[u8;2])->u8{match x{[N,N]=>1,[N,_]=>2,_=>3}}fn main(){check(f([7,7])==1&&f([7,0])==2&&f([0,7])==3);}