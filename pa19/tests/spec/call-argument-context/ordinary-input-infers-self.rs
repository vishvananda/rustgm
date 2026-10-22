trait C{type Out;fn get(x:Self)-><Self as C>::Out;}impl C for u8{type Out=u16;fn get(x:u8)->u16{x as u16}}fn main(){let _:u16=<_ as C>::get(73u8);}
