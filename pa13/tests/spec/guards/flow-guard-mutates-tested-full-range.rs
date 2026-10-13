fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn entry()->u64{let mut x=7u8;match x{0..=255 if {x=8;false}=>99,_=>x as u64}}fn main(){check(entry()==8);}