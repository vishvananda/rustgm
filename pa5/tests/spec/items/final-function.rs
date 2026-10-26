#![feature(final_associated_functions)]
#![allow(dead_code,internal_features)]
trait T{final fn value(&self)->u8{7}}
impl T for u8{}
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{T::value(&0u8)}
