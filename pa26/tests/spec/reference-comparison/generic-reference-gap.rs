fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn compare<T:core::cmp::PartialOrd>(a:&T,b:&T)->bool{a<b}fn main(){check(compare(&3u8,&7u8));check(!compare(&3u8,&3u8));}