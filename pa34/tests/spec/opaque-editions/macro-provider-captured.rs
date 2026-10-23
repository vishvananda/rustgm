#[macro_export]macro_rules! make{($ret:ty)=>{fn op<'a>(x:&'a u8)->$ret{x}}}
