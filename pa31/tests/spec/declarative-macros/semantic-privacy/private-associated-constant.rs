#![feature(decl_macro)]
mod provider { pub struct Record; impl Record {const VALUE:u32=7;} pub macro value() {Record::VALUE} }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 {provider::value!()}
