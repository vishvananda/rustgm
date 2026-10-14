macro_rules! d{()=>{use core::ops::*;impl Drop for S{fn drop(&mut self){}}}}d!();struct S;fn main(){let _s=S;}
