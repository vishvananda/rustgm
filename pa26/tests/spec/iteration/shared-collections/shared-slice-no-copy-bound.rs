fn check(b:bool){if !b{let z=0;let _=1/z;}}struct V(u8);fn count<T>(a:&[T])->u8{let mut n=0u8;for _ in a{n+=1u8;}n}fn main(){check(count(&[V(1),V(2)])==2);}
