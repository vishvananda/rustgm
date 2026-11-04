#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(diagnostic_on_const,diagnostic_on_unmatched_args)]
trait T{fn value(&self)->u8;}#[diagnostic::on_const(message="not constant")]impl T for u8{fn value(&self)->u8{7}}#[diagnostic::on_unmatched_args(note="expected empty arguments")]macro_rules! value{()=>{9u8}}#[unsafe(no_mangle)]pub extern "C" fn observed_trait()->u8{7u8.value()}#[unsafe(no_mangle)]pub extern "C" fn observed_macro()->u8{value!()}
