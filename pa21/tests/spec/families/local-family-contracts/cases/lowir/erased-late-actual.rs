trait Q{type Out<'a> where 'a:'static;fn f<'a>(x:u8);}impl Q for u8{type Out<'a>=u8 where 'a:'static;fn f<'a>(x:Self::Out<'a>) {}}
fn main(){}
