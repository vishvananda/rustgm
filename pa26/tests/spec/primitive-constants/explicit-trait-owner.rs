fn check(b:bool){if !b{let z=0;let _=1/z;}}trait K{const MAX:u8;}impl K for u32{const MAX:u8=7;}fn main(){check(<u32 as K>::MAX==7);}
