extern "C" fn f(x:i32)->i32{x}fn take<F:Fn(i32)->i32>(f:F){let _=f(3);}fn main(){let p:extern "C" fn(i32)->i32=f;take(p);}
