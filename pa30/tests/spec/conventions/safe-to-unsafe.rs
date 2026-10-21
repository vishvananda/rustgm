extern "C" fn f(x:i32)->i32{x}fn main(){let p:unsafe extern "C" fn(i32)->i32=f;let _=unsafe{p(3)};}
