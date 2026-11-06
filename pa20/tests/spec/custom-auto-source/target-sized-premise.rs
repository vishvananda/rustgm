#![feature(auto_traits)]
auto trait Auto{}impl<T:?Sized> Auto for T where T:Sized{}
