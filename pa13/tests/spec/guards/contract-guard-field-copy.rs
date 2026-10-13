fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn entry()->u64{let x=N(7);match x{n if n.0==7=>n.0 as u64,_=>2}}
fn main(){check(entry()==7);}