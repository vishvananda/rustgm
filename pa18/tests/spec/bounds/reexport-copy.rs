mod m{pub use core::marker::Copy as C;}fn f<T:m::C>(x:T)->[T;2]{[x;2]}
fn main(){}
