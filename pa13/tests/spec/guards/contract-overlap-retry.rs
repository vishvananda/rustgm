fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn entry()->u64{let mut c=0;match 7u8{0..=9|5..=15|_ if {c+=1;false}=>99,_=>c}}
fn main(){check(entry()==3);}