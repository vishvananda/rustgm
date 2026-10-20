trait Q<T>{type Out<'a> where T:'a;fn f<'a>(x:u8);}impl<T>Q<T>for u8{type Out<'a>=u8 where T:'a;fn f<'a>(x:Self::Out<'a>) {}}
fn main(){}
