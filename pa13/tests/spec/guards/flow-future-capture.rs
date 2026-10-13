fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn entry()->u64{let mut x=(7u8,1u8);match x{_ if {x.1=2;false}=>99,(_,n)=>n as u64}}fn main(){check(entry()==2);}