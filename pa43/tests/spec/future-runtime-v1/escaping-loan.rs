async fn value(x:&mut u32){*x+=1;} fn main(){let future;{let mut x=1;future=value(&mut x);}drop(future);}
