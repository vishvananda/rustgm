#![feature(staged_api,const_trait_impl)]
#![stable(feature="root_api",since="1.0.0")]
struct Value;const impl Value{pub fn read()->u8{7}}const SEVEN:u8=Value::read();fn main(){}
