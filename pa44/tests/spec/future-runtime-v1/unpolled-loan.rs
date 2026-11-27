async fn value(x:&mut u32){*x+=1;} fn main(){let mut x=1;let future=value(&mut x);let _observed=x;drop(future);}
