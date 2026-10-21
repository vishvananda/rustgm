fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn relation<T:PartialOrd>(a:&T,b:&T)->Option<core::cmp::Ordering>{a.partial_cmp(b)}fn main(){let n=0.0f64/0.0;check(match relation(&n,&n){None=>true,_=>false});check(!(n<1.0));check(!(n<=1.0));check(!(n>1.0));check(!(n>=1.0));}
