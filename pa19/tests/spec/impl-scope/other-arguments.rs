fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait Show<A>{fn show(&self)->u8;}impl Show<u16> for u8{fn show(&self)->u8{7}}}struct E;impl m::Show<u8> for E{fn show(&self)->u8{3u8.show()}}fn main(){check(<E as m::Show<u8>>::show(&E)==7);}
