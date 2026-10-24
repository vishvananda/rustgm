#![feature(const_trait_impl,const_destruct)]
pub const trait Consume:Copy{fn consume(self){}}
impl Consume for u8{}
#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{<u8 as Consume>::consume(7);7}
