fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
fn sum(a:(u64,u64),b:(u64,u64))->u64{a.0+a.1+b.0+b.1} fn main(){let mut p=(1,2);let n=sum(p,{p.0=9;p});check(n==14);}
