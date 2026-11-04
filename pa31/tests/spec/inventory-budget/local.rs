#![feature(staged_api,allow_internal_unstable)]
#![unstable(feature="inventory_api",issue="none")]
#[allow_internal_unstable()]
macro_rules! route { () => { 7u8 }; }
pub fn value()->u8{route!()}
