fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn compare<T:core::cmp::PartialOrd>(a:&T,b:&T)->bool{a<b}fn main(){let _=compare(&1u8,&2u8);}