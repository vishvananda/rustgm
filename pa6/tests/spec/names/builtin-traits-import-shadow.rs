mod m { pub trait C {} }
use m::C as Copy;
fn f<T: Copy>() {}
