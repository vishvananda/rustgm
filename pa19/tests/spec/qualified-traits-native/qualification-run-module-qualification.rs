fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}fn main(){let x=<u8 as m::A>::f(3);check(x==3);}