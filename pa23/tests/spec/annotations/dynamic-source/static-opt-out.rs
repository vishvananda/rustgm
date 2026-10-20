trait T{fn f<A>() where Self:Sized;}fn check(_: &dyn T){}
fn main(){}
