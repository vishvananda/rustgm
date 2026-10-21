fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f:fn(&f32,&f32)->Option<core::cmp::Ordering>=f32::partial_cmp;let n=0.0f32/0.0;check(match f(&n,&n){None=>true,_=>false});}
