#![feature(allow_internal_unsafe)]
unsafe fn target()->u8{7}
#[allow_internal_unsafe]macro_rules! call{()=>{target()}}pub fn test()->u8{call!()}
