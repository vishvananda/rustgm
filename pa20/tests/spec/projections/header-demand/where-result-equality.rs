trait It{type Item;}
trait IntoIt{type Item;type IntoIter:It<Item=Self::Item>;}
trait Tr{}
struct W<I>(I);
impl<I> Tr for W<I> where I:It<Item:IntoIt>,<<I::Item as IntoIt>::IntoIter as It>::Item:Copy{}
fn main(){}
