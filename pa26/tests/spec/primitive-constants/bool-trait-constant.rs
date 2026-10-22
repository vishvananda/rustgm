fn check(b:bool){if !b{let z=0;let _=1/z;}}trait K{const MAX:u32;}impl K for bool{const MAX:u32=7;}fn main(){check(bool::MAX==7);}
