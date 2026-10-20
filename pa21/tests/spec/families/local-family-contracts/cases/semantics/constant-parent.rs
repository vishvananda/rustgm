trait Q<P:Copy>{type Out<T>;const C:Self::Out<P>;}impl<P:Copy>Q<P> for u8{type Out<T>=u8;const C:u8=0;}
fn main(){}
