fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn require<A:?Sized+core::cmp::PartialOrd<B>,B:?Sized>(){}fn main(){require::<&u8,&u8>();}