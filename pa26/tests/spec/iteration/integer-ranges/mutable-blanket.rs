fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let mut r=1u8..5u8;let mut n=0u8;for x in &mut r{n+=x;if x==2{break;}}check(n==3);match r.next(){Some(x)=>check(x==3),None=>check(false)}}
