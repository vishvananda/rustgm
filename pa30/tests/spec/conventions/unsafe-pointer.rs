extern "C" fn f(x:i32)->i32{x}fn main(){let p:extern "C" fn(i32)->i32=f;let q:unsafe extern "C" fn(i32)->i32=p;let _=unsafe{q(3)};}
