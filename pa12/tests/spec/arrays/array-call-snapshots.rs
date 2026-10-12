fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
fn sum(a:[u64;2],b:[u64;2])->u64{a[0]+a[1]+b[0]+b[1]}fn main(){let mut a=[1,2];let s=sum(a,{a[1]=9;a});check(s==13);}
