fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn entry()->u64{let y=N(7);match 7u8{_ if let n=y&&true=>n.0 as u64,_=>99}}fn main(){check(entry()==7);}