#![feature(const_trait_impl)]
struct Value;macro_rules! make{()=>{const impl Value{fn get()->u8{7}}};}make!();const SEVEN:u8=Value::get();fn main(){}
