trait It{type Item;}
trait IntoIt{type Item;type IntoIter:It<Item=Self::Item>;}
unsafe trait TrustedLen{}
struct FlattenCompat<I,U>(I,U);
struct Flatten<I>(I);
unsafe impl<I> TrustedLen for Flatten<I> where I:It,FlattenCompat<I,<I::Item as IntoIt>::IntoIter>:TrustedLen{}
fn main(){}
