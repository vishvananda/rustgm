trait T{}impl T for u8{}fn f(x:&mut u8)->&dyn T{x}
fn main(){}
