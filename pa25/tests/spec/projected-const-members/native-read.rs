#![feature(const_trait_impl)]
pub const trait Read<R>{fn read(value:R)->u8;}
pub const trait Maker{type Output:[const]Read<u8>;}
pub const fn read<M:[const]Maker>(value:u8)->u8{<M::Output as Read<u8>>::read(value)}
pub struct Tag;pub struct Out;
const impl Maker for Tag{type Output=Out;}
const impl Read<u8> for Out{fn read(value:u8)->u8{value}}
pub const VALUE:u8=read::<Tag>(41u8);
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{read::<Tag>(value)}
fn main(){}
