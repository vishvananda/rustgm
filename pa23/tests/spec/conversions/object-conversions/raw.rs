trait T{}impl T for u8{}fn f(x:*const u8)->*const dyn T{x}
fn main(){}
