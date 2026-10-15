fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{7}}}fn g<T:m::A>(x:T)->u8{x.f()}fn main(){let x=g(3u8);check(x==7);}