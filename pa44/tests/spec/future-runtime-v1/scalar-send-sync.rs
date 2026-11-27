fn require<T:Send+Sync>(_:T){} async fn value(_:u32){} fn main(){require(value(1));}
