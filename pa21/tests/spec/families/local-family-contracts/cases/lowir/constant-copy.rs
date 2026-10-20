trait Q{type Out<T:Copy>;const C:Self::Out<u8>;}impl Q for u8{type Out<T:Copy>=u8;const C:u8=0;}
fn main(){}
