fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait Show{fn show(&self)->u8;}impl Show for u8{fn show(&self)->u8{7}}}struct R;impl R{fn show(&self)->u8{7}}impl m::Show for R{fn show(&self)->u8{9}}struct E;impl m::Show for E{fn show(&self)->u8{R.show()}}fn main(){check(<E as m::Show>::show(&E)==7);}
