fn check(b:bool){if !b{let z=0;let _=1/z;}}trait K{const MAX:u32;fn get()->u32{Self::MAX}}impl K for u32{const MAX:u32=7;}fn main(){check(<u32 as K>::get()==7);}
