pub async fn scalar(x:u8)->u8{x}
pub async fn raw(_: *mut u8)->u8{7}
pub async fn output()->*mut u8{0usize as *mut u8}
pub async fn generic<T>(x:T)->T{x}
pub async fn borrowed<'a>(x:&'a mut u8)->u8{*x}
