#![feature(const_closures, const_trait_impl, const_destruct)]
pub const ANSWER:u8={let call=|value:u8| value+1;call(22)};
