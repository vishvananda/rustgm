trait Q{type Out<'a> where Self:'a;}struct Hold<'a,T:Q>(T::Out<'a>);
fn main(){}
