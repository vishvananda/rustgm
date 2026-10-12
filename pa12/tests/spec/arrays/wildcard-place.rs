fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
struct N(u64);fn main(){let a=[N(5),N(7)];let mut n=0;let _=a[{n+=1;1}];check(n==1 && a[0].0==5 && a[1].0==7);}
