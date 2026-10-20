trait Q{type Out<T:'static>;}struct Hold<U:Q,T>(U::Out<T>);
fn main(){}
