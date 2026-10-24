#![feature(decl_macro)]
mod provider { trait Hidden {fn read(&self)->u32;} pub struct Record; impl Hidden for Record {fn read(&self)->u32{7}} pub fn make()->impl Hidden {Record} pub macro value() {{let _=make();7u32}} }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 {provider::value!()}
