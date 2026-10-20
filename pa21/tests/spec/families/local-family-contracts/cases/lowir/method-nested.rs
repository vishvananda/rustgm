trait Q{type Out<'a>;fn f()->for<'a>fn(&'a u8)->Self::Out<'a>;}impl Q for u8{type Out<'a>=&'a u8;fn f()->for<'a>fn(&'a u8)->&'a u8{loop{}}}
fn main(){}
