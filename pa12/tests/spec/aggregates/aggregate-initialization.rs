fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
struct S(u64);fn main(){let mut n=0;let x;while n<1{n+=1;}if n==1{x=S(7);}else{x=S(3);}check(x.0==7);}
