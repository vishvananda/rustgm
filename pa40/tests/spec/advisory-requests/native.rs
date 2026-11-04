#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(intra_doc_pointers,strict_provenance_lints)]
/// See [`pointer::is_null`].
#[unsafe(no_mangle)]pub extern "C" fn observed_docs()->u8{7}#[unsafe(no_mangle)]pub extern "C" fn observed_pointer()->u8{let value:u8=9;let p=&value as *const u8;let address=p as usize;let q=address as *const u8;unsafe{*q}}
