#![feature(const_trait_impl)]
pub const trait Base<Rhs>{fn read(&self,rhs:&Rhs)->u8;}
pub const trait Probe<X>{fn check<Y>(x:&X,y:&Y)->u8 where X:[const]Base<Y>{Base::read(x,y)}}
pub struct Host;const impl Probe<u8> for Host{}
const impl Base<u8> for u8{fn read(&self,rhs:&u8)->u8{*rhs}}
pub const VALUE:u8=<Host as Probe<u8>>::check(&7u8,&41u8);
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
#[unsafe(no_mangle)]pub extern "C" fn probe(x:u8)->u8{<Host as Probe<u8>>::check(&7u8,&x)}
fn main(){}
