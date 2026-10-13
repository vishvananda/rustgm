fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn entry()->u64{let y=N(7);match 7u8{7|_ if let n=y&&{return n.0 as u64;}=>99,_=>99}}fn main(){check(entry()==7);}