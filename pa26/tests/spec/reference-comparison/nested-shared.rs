fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn compare<T:core::cmp::Ord>(a:T,b:T)->bool{a<b}fn main(){let a=3u8;let b=7u8;check(compare(&&&a,&&&b));}