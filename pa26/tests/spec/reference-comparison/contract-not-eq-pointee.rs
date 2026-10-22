fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct A;fn require<T:core::cmp::Eq>(){}fn main(){require::<&A>();}