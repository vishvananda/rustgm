trait Has:'static{type Item;}trait A{type X<'a>;type Y<'a>:Has<Item=Self::X<'a>>;}
fn main(){}
