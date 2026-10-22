fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn require<A:core::cmp::Ord>(){}fn main(){require::<&mut u8>();}