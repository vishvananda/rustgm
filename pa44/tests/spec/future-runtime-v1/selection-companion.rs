pub async fn value()->u8{7}
pub async fn wide()->u16{9}
pub async fn require_future<F:Future<Output=u8>>(_:F)->u8{7}
