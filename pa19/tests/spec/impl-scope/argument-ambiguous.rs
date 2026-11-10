fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait Show{fn show(&self,x:u8)->u8;}impl Show for u8{fn show(&self,x:u8)->u8{x}}}trait Other{fn show(&self,x:u16)->u8;}impl Other for u8{fn show(&self,x:u16)->u8{9}}struct E;impl m::Show for E{fn show(&self,x:u8)->u8{3u8.show(x)}}fn main(){}
