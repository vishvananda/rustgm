unsafe fn target()->u8{7}
macro_rules! call{()=>{unsafe{target()}}}pub fn test()->u8{call!()}
