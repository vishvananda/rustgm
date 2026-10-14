mod m { pub use core::marker::Copy as C; }
use m::*;
fn f<T: C>() {}
