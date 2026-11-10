fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait Show{fn show(&self)->u8;}impl Show for u8{fn show(&self)->u8{7}}}trait Other{fn show(&self)->u16;}impl Other for u8{fn show(&self)->u16{9}}struct E;impl m::Show for E{fn show(&self)->u8{let value:u8=3u8.show();value}}fn main(){check(<E as m::Show>::show(&E)==7);}
