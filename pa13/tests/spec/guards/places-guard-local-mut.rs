fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn entry()->u64{match 7u8{_ if let mut n=7u8&&{n+=1;true}=>n as u64,_=>99}}fn main(){check(entry()==8);}