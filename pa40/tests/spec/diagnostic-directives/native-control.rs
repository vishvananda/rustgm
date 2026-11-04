#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
trait T{fn value(&self)->u8;}impl T for u8{fn value(&self)->u8{7}}macro_rules! value{()=>{9u8}}#[unsafe(no_mangle)]pub extern "C" fn observed_trait()->u8{7u8.value()}#[unsafe(no_mangle)]pub extern "C" fn observed_macro()->u8{value!()}
