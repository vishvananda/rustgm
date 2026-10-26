#![allow(dead_code,internal_features)]
trait T{fn value<U>(&self,_:U)->u8{7}}
fn call(v:&dyn T)->u8{v.value(0u8)}
