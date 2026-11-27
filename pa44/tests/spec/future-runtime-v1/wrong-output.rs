use core::future::Future; fn require<T:Future<Output=u8>>(_:T){} async fn value()->u32{1} fn main(){require(value());}
