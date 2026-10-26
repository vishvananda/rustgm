#![allow(dead_code,internal_features)]
trait T{fn value(&self)->u8{1}}
impl T for u8{fn value(&self)->u8{9}}
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{T::value(&0u8)}
