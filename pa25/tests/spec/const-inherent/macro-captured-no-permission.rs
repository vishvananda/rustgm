#![feature(allow_internal_unstable)]struct Value;#[allow_internal_unstable(const_trait_impl)]macro_rules! make{($q:tt)=>{$q impl Value{fn read()->u8{7}}};}make!(const);fn main(){}
