#![feature(const_trait_impl)]
struct Value(u8);
const impl Value { fn new(v:u8)->Self{Self(v)} fn get(&self)->u8{self.0} }
const SEVEN:u8=Value::new(7).get(); fn main(){}
