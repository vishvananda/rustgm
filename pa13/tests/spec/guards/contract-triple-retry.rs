fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn entry()->u64{let mut c=0;match (0u8,0u8,0u8){(0|_,0|_,0|_) if {c+=1;false}=>99,_=>c}}
fn main(){check(entry()==8);}