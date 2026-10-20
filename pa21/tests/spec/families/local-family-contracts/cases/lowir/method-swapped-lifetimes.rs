trait Q{type Out<'a,'b>;fn f()->for<'a,'b>fn(&'a u8,&'b u8)->Self::Out<'a,'b>;}impl Q for u8{type Out<'a,'b>=(&'a u8,&'b u8);fn f()->for<'a,'b>fn(&'a u8,&'b u8)->(&'b u8,&'a u8){loop{}}}
fn main(){}
