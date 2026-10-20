trait Has{type Item;}trait A{type X<'a>:Has<Item=Self::Y<'a>>;type Y<'a>:Has<Item=Self::X<'a>>;}
fn main(){}
