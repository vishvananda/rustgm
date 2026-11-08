trait It{type Item;}
trait IntoIt{type Item;type IntoIter:It<Item=Self::Item>;}
impl<I:It> IntoIt for I{type Item=I::Item;type IntoIter=I;}
trait Len:It{}
struct Flat<I>(I);
impl<I,U> It for Flat<I> where I:It<Item:IntoIt<IntoIter=U,Item=U::Item>>,U:It{type Item=U::Item;}
impl<I> Len for Flat<I> where I:It<Item:IntoIt>{}
fn main(){}
