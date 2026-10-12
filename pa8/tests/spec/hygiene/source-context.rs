macro_rules! make{($e:expr)=>{fn f(x:i32)->i32{$e}}} make!(x);
