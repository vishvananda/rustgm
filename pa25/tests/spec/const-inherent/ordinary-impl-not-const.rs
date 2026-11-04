#![feature(const_trait_impl)]
struct Value;impl Value{fn get()->u8{7}}const BAD:u8=Value::get();fn main(){}
