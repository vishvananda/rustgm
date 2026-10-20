mod m{pub trait T{fn get(&self)->u8;}}fn f(x:&dyn m::T)->u8{x.get()}
fn main(){}
