fn check(b:bool){if !b{let z=0;let _=1/z;}}trait K{const MAX:u32;fn get()->u32;}impl K for u32{const MAX:u32=7;fn get()->u32{Self::MAX}}fn main(){check(<u32 as K>::get()==4294967295u32);}
