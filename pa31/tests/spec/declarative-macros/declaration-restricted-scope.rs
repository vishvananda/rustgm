#![feature(decl_macro)]
mod provider{pub(in crate::provider) macro value(){not_resolved()}mod child{use super::value as selected;}}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
