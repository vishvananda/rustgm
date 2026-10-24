#![feature(decl_macro)]
mod provider { struct Hidden {pub value:u32} pub fn make()->Hidden {Hidden{value:7}} pub macro value() {make().value} }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 {provider::value!()}
