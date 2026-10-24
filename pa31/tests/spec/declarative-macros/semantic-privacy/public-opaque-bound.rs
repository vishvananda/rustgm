#![feature(decl_macro)]
mod provider {pub trait Read {fn read(&self)->u32;} struct Hidden; impl Read for Hidden{fn read(&self)->u32{7}} pub fn make()->impl Read {Hidden} pub macro value(){make()} }
use provider::Read;
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{provider::value!().read()}
