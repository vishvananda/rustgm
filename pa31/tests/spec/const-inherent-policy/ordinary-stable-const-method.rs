#![feature(staged_api,const_trait_impl)]
#![stable(feature="root_api",since="1.0.0")]
#[stable(feature="root_api",since="1.0.0")]pub struct Value;impl Value{#[stable(feature="root_api",since="1.0.0")]#[rustc_const_stable(feature="method_api",since="1.0.0")]pub const fn read()->u8{7}}#[stable(feature="root_api",since="1.0.0")]#[rustc_const_stable(feature="caller_api",since="1.0.0")]pub const fn caller()->u8{Value::read()}fn main(){}
