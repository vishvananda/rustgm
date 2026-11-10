fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait Show{fn show<T:Copy>(&self,x:T)->u8;}impl Show for u8{fn show<T:Copy>(&self,x:T)->u8{7}}}struct N;struct E;impl m::Show for E{fn show<T:Copy>(&self,x:T)->u8{3u8.show(N)}}fn main(){}
