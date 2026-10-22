fn check(b:bool){if !b{let z=0;let _=1/z;}}use std::clone::Clone as C;fn main(){let a=[7u8];let mut i=C::clone(&a.iter());match i.next(){Some(x)=>check(*x==7),None=>check(false)}}
