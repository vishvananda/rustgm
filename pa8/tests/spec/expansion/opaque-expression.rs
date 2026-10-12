macro_rules! a{($e:expr)=>{$e*3}} macro_rules! b{($e:expr)=>{a!($e)}} fn f()->i32{b!(1+2)}
