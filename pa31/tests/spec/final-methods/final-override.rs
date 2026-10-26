#![feature(final_associated_functions)]
#![allow(dead_code,internal_features)]
trait T{final fn value(&self)->u8{7}}
impl T for u8{fn value(&self)->u8{9}}
