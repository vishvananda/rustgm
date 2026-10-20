trait T{}impl T for u8{}fn f(x:&u8)->&mut dyn T{x}
fn main(){}
