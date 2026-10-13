fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn entry()->u64{let x:u8;match x{_ if true=>1,_=>2}}
fn main(){check(entry()==1);}