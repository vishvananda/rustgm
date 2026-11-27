pub struct NonCopy;
pub async fn copy_bound<T:Copy>(x:T)->T{x}
pub async fn related<'a,'b:'a>(x:&'a mut u8,_:&'b ())->u8{*x}
