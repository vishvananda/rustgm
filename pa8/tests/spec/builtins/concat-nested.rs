macro_rules! m{($e:expr)=>{concat!($e,concat!("b","c"))}} fn f()-> &'static str{m!("a")}
