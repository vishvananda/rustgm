#![feature(decl_macro)]
use provider::value as selected;mod provider{pub macro value(){not_resolved()}}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
