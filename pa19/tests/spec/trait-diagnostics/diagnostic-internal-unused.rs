#![allow(dead_code,unused_variables,internal_features)]
#![feature(rustc_attrs)]
#[rustc_on_unimplemented(message="NEEDS {Self}",label="LABEL {Self}",note="NOTE")]trait Need{}
fn main(){}
