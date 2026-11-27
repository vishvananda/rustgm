#![feature(rustc_attrs)]
#[rustc_doc_primitive="not_a_primitive"]mod docs{pub fn answer()->u64{7}}#[unsafe(no_mangle)]pub extern "C" fn native_observed()->u64{docs::answer()}
