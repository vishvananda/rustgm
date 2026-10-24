#![feature(decl_macro)]
mod provider {pub struct Record {pub value:u32} pub fn make()->(Record,) {(Record{value:7},)} pub macro value(){make().0.value} }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{provider::value!()}
