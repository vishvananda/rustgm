fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(Ord::min(-3i32,7)==-3);check(Ord::max(3u64,7)==7);check(Ord::clamp(9u8,2,5)==5)}