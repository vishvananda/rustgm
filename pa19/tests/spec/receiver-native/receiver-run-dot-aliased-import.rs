fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self}}}use m::A as Alias;fn main(){let x=7u8.f();check(x==7);}