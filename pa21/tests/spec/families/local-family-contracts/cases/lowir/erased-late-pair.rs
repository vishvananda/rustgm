trait Q{type Out<'a,'b> where 'a:'b;fn f<'a,'b>(x:u8);}impl Q for u8{type Out<'a,'b>=u8 where 'a:'b;fn f<'a,'b>(x:Self::Out<'a,'b>) {}}
fn main(){}
