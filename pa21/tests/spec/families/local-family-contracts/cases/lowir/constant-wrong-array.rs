trait Q{type Out<const N:usize>;const C:Self::Out<3>;}impl Q for u8{type Out<const N:usize>=[u8;N];const C:[u8;4]=[0;4];}
fn main(){}
