fn require<T:Send>(_:T){} async fn value(_: *mut u8){} fn main(){require(value(core::ptr::null_mut()));}
