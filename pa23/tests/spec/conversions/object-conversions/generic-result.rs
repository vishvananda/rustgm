trait T{type A;}impl T for u8{type A=u16;}fn f<X:T>(x:&X)->&dyn T<A=X::A>{x}
fn main(){}
