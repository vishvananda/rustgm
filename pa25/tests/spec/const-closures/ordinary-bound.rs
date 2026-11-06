#![feature(const_closures, const_trait_impl, const_destruct)]
const fn through<F:Fn(u8)->u8>(call:F,value:u8)->u8 {call(value)}
pub const ANSWER:u8=through(const |value:u8| value+1,22);
