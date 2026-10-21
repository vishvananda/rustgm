fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn relation<T:PartialOrd>(a:&T,b:&T)->Option<core::cmp::Ordering>{a.partial_cmp(b)}fn main(){check(match relation(&-0.0f64,&0.0){Some(core::cmp::Ordering::Equal)=>true,_=>false});check(match relation(&(1.0f64/0.0),&1.0){Some(core::cmp::Ordering::Greater)=>true,_=>false});}
