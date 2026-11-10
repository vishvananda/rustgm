fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod m{pub trait Show{fn show(&self)->u8;}impl Show for u8{fn show(&self)->u8{7}}}trait Other{fn other(&self)->u8;}struct E;impl Other for E{fn other(&self)->u8{3u8.show()}}fn main(){}
