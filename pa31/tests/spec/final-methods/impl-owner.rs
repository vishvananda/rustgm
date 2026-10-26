#![feature(final_associated_functions)]
#![allow(dead_code,internal_features)]
trait T{fn value(&self)->u8;}impl T for u8{final fn value(&self)->u8{7}}
