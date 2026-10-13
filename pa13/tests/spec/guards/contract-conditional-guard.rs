fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn entry()->u64{match (0u8,7u8){x if let (0,n)=x=>n as u64,_=>99}}
fn main(){check(entry()==7);}