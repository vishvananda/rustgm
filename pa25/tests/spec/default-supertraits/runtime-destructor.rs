#![feature(const_trait_impl,const_destruct)]
use core::marker::Destruct;
pub const trait Base{fn read(&self)->u8;}
pub const trait Derived:[const]Base{fn through(&self)->u8 where Self:[const]Destruct{Base::read(self)}}
pub struct Value(u8);
const impl Base for Value{fn read(&self)->u8{self.0}}
impl Drop for Value{fn drop(&mut self){}}
const impl Derived for Value{}
pub const VALUE:u8=Derived::through(&Value(41));
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
fn main(){}
