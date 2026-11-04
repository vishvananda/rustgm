#![feature(const_trait_impl)]
struct Value;#[cfg(any())]const impl Value{const fn get()->u8{7}}const impl Value{fn get()->u8{7}}const SEVEN:u8=Value::get();fn main(){}
