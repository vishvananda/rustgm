fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait Parent{fn parent(&self)->u8;}pub trait Show:Parent{fn show(&self)->u8;}impl Parent for u8{fn parent(&self)->u8{7}}}struct E;impl m::Parent for E{fn parent(&self)->u8{7}}impl m::Show for E{fn show(&self)->u8{self.parent()}}fn main(){check(<E as m::Show>::show(&E)==7);}
