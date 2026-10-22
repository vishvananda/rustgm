fn check(b:bool){if !b{let z=0;let _=1/z;}}type X=u32;trait K{const MAX:u32;fn get()->u32;}impl K for X{const MAX:u32=7;fn get()->u32{Self::MAX}}fn main(){check(<X as K>::get()==4294967295u32);}
