#![feature(const_trait_impl)]
pub const trait Base<R>{fn make(value:R)->Self;}
pub const trait Maker<R>{type Output:[const]Base<R>;}
pub const fn convert<M:[const]Maker<R>,R>(value:R)->M::Output{Base::make(value)}
pub struct Tag;pub struct Out(u8);
const impl Maker<u8> for Tag{type Output=Out;}
const impl Base<u8> for Out{fn make(value:u8)->Self{Out(value)}}
pub const VALUE:u8=convert::<Tag,u8>(41u8).0;
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{convert::<Tag,u8>(value).0}
fn main(){}
