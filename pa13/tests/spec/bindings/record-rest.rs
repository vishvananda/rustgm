fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
mod m{pub struct S{a:u8,pub b:u64}pub fn make()->S{S{a:1,b:7}}}fn main(){let m::S{b,..}=m::make();check(b==7);}