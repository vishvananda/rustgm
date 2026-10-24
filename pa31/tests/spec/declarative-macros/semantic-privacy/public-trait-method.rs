#![feature(decl_macro)]
mod provider {pub trait Read {fn read(&self)->u32;} pub struct Record; impl Read for Record{fn read(&self)->u32{7}} pub macro value($x:expr) {$x.read()} }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{let r=provider::Record;provider::value!(r)}
