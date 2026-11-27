fn require<T:Unpin>(_:T){} async fn value()->u32{1} fn main(){require(value());}
