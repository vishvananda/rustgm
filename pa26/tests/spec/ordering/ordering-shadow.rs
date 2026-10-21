fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod core{pub mod cmp{pub enum Ordering{Other}}}use ::core::cmp as c;fn main(){check(c::Ordering::Less as i8 == -1);check(core::cmp::Ordering::Other as i8 == 0);}
