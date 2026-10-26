#![feature(final_associated_functions)]
#![allow(dead_code,internal_features)]
trait T{final fn value<U>(&self,_:U)->u8{7}}
impl T for u8{}
fn call(v:&dyn T)->u8{v.value(0u8)}
