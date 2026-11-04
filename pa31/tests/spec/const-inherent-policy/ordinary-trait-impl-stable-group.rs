#![feature(staged_api,const_trait_impl)]
#![stable(feature="root_api",since="1.0.0")]
#[stable(feature="root_api",since="1.0.0")]pub struct Value;
trait Give{fn read()->u8;}#[rustc_const_stable(feature="impl_api",since="1.0.0")]impl Give for Value{fn read()->u8{7}}fn main(){}
