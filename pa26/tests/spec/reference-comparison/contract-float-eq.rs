fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn require<T:core::cmp::Eq>(){}fn main(){require::<&f32>();}