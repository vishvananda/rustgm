mod m{pub struct S{pub a:u8,b:u8}pub fn make()->S{S{a:1,b:2}}}fn main(){let s=m::S{a:3,..m::make()};}
