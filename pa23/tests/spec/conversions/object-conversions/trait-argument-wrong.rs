trait T<A>{}impl T<u8> for u16{}fn f(x:&u16)->&dyn T<u32>{x}
fn main(){}
