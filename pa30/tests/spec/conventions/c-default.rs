extern fn f(x:i32)->i32{x}fn main(){let p:extern "C" fn(i32)->i32=f;let _=p(3);}
