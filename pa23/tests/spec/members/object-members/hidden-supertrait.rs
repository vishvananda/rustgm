mod m{pub trait P{fn get(&self)->u8;}pub trait T:P{}}fn f(x:&dyn m::T)->u8{x.get()}
fn main(){}
