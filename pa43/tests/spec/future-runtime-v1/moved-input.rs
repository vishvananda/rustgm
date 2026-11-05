struct Token(u32); async fn value(_:Token){} fn main(){let token=Token(7);let future=value(token);let _observed=token.0;drop(future);}
