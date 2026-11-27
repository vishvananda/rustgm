#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
trait T{fn value(&self)->u8;}impl T for u8{fn value(&self)->u8{9}}#[unsafe(no_mangle)]pub extern "C" fn observed_cfg()->u8{7}#[unsafe(no_mangle)]pub extern "C" fn observed_notable()->u8{9u8.value()}
