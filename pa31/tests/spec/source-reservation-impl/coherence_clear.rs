#![feature(rustc_attrs)]
trait A{}trait B{}impl<T:A>B for T{}impl B for u8{}
