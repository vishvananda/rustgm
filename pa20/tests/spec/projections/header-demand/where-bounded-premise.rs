trait It{type Item;}
trait IntoIt{type Item;type IntoIter:It<Item=Self::Item>;}
unsafe trait TrustedLen{}
struct FlattenCompat<I,U>(I,U);
struct Flatten<I:It<Item:IntoIt>>(FlattenCompat<I,<I::Item as IntoIt>::IntoIter>);
unsafe impl<I> TrustedLen for Flatten<I> where I:It<Item:IntoIt>,FlattenCompat<I,<I::Item as IntoIt>::IntoIter>:TrustedLen{}
fn main(){}
