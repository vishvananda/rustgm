#![feature(const_trait_impl)]
struct Value(u8);
const impl Value { fn new(v:u8)->Self{Self(v)} fn get(&self)->u8{self.0} }
fn main(){let value=Value::new(9);let _=value.get();}
