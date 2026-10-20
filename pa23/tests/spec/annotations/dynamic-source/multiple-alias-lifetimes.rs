trait T<'a,'b>:'a+'b{}type A<'a,'b>=dyn T<'a,'b>;
fn main(){}
