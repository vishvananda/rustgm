#![feature(const_closures, const_trait_impl, const_destruct)]
const fn through<F:[const]FnMut(u8)->u8>(mut call:F,value:u8)->u8 where F:[const]core::marker::Destruct {call(value)}
const fn compute(value:u8)->u8 {through(const move |extra:u8| value+extra,2)}
pub const ANSWER:u8=compute(21);
