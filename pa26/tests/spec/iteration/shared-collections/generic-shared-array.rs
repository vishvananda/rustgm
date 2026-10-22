fn check(b:bool){if !b{let z=0;let _=1/z;}}fn count<T,const N:usize>(a:&[T;N])->u8{let mut n=0u8;for _ in a{n+=1u8;}n}fn main(){check(count(&[1u8,2])==2);}
