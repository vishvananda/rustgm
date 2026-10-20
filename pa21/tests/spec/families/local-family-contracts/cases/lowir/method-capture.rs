struct W<'p>(&'p u8);trait Q{type Out<'a> where Self:'a;fn f<'a>(x:&'a Self)->Self::Out<'a>;}impl<'p>Q for W<'p>{type Out<'a>=&'a &'p u8 where Self:'a;fn f<'b>(x:&'b Self)->&'b &'p u8{&x.0}}
fn main(){}
