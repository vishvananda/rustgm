fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let r=1u8..3u8;let s=r.clone();let mut a=0u8;for x in r{a+=x;}for x in s{a+=x;}check(a==6);}
