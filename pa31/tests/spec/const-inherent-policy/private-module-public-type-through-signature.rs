#![feature(staged_api,const_trait_impl)]
#![stable(feature="root_api",since="1.0.0")]
mod hidden{#[stable(feature="root_api",since="1.0.0")]pub struct Value;const impl Value{#[stable(feature="root_api",since="1.0.0")]pub fn read()->u8{7}}}#[stable(feature="root_api",since="1.0.0")]pub fn value()->hidden::Value{hidden::Value}fn main(){}
