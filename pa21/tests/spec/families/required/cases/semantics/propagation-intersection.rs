trait Has{type Item;}trait A{type X<'a>;type Y<'a>:Has<Item=Self::X<'a>>where Self:'a;fn f<'a>(&'a self)->Self::Y<'a>;fn g<'a>(x:Self::X<'a>);}
fn main(){}
