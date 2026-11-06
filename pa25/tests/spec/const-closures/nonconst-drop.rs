#![feature(const_closures, const_trait_impl, const_destruct)]
struct Owned;
impl Drop for Owned {fn drop(&mut self){}}
pub const ANSWER:u8={let value=Owned;let call=const move || {let _whole=value;23u8};call()};
