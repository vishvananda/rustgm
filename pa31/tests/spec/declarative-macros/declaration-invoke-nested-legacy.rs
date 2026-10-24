#![feature(decl_macro)]
pub macro selected(){7u32}macro_rules! outer{()=>{selected!()};}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{outer!()}
