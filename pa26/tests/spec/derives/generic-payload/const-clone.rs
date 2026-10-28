#![feature(derive_const,const_trait_impl,const_clone)]
#[derive_const(Clone)]struct Wrap<T>(T);
const VALUE:Wrap<u8>=Wrap(7).clone();fn main(){println!("{}",VALUE.0);}
