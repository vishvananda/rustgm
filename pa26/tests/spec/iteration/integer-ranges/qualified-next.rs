fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let mut r=2u16..4u16;match <core::ops::Range<u16> as Iterator>::next(&mut r){Some(x)=>check(x==2),None=>check(false)}check(r.start==3&&r.end==4);}
