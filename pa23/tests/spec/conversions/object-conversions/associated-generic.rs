trait T{type A;}fn f<X:T<A=u16>>(x:&X)->&dyn T<A=u16>{x}
fn main(){}
