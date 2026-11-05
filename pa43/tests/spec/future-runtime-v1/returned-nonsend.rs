fn require<T:Send+Sync>(_:T){} async fn value()->*mut u8{0usize as *mut u8} fn main(){require(value());}
