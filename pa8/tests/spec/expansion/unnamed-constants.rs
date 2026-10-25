#![allow(dead_code,unused_imports)]
macro_rules! twice {($i:item)=>{$i $i}}
twice!(const _: () = (););
fn main(){twice!(const _: u8=1;);}
