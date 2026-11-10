fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait CastInto<T:Copy>:Copy{fn cast(self)->T;}impl CastInto<i16> for u8{fn cast(self)->i16{self as i16}}pub trait Float{type Int:CastInto<i16>;}pub struct F;impl Float for F{type Int=u8;}}fn conv<T:m::Float>(x:T::Int)->i16{x.cast()}fn main(){check(conv::<m::F>(7)==7);}
