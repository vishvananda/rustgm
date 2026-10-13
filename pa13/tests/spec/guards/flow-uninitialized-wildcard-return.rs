fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn entry()->u64{let x:u8;match x{_ if return 7=>99,_=>0}}fn main(){check(entry()==7);}