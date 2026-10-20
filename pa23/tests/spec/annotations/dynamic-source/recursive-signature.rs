trait T{fn f(&self)->&dyn T;}fn check(_: &dyn T){}
fn main(){}
