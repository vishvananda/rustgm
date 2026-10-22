trait Other{fn next(&mut self)->u8;}impl Other for core::ops::Range<u8>{fn next(&mut self)->u8{7}}fn main(){let mut r=1u8..4u8;let _=r.next();}
