macro_rules! m{($e:expr)=>{stringify!($e)}} fn f()-> &'static str{m!(1 +2)}
