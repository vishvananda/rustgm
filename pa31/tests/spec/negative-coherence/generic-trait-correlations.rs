#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
trait A<X:?Sized>{}impl<T:?Sized>!A<T> for &T{}#[rustc_strict_coherence]trait B<X:?Sized>{}impl<T,X:?Sized>B<X> for T where T:A<X>{}impl<T:?Sized>B<T> for &T{}fn main(){}
