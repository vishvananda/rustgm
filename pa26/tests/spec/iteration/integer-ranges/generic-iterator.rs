fn check(b:bool){if !b{let z=0;let _=1/z;}}fn sum<I:Iterator<Item=u8>>(i:I)->u8{let mut n=0u8;for x in i{n+=x;}n}fn main(){check(sum(1u8..4u8)==6);}
