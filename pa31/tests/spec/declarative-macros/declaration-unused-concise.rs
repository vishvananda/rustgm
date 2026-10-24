#![feature(decl_macro)]
pub macro unused($value:expr){not_resolved_until_invocation($value)}
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
