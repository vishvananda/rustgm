#![feature(const_trait_impl)]
pub const trait Base{fn read(&self)->u8;}
pub const trait Derived:[const]Base{fn through(&self)->u8{Base::read(self)}}
pub struct Value(u8);
const impl Base for Value{fn read(&self)->u8{self.0}}
const impl Derived for Value{}
pub const VALUE:u8=Derived::through(&Value(41));
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
fn main(){}
