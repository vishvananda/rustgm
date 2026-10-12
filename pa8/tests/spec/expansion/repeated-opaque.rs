macro_rules! a{($e:expr)=>{($e)+($e)}} fn f()->i32{a!(1+2)}
