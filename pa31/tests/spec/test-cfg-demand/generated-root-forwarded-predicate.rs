#![allow(dead_code,unused_macros)]
macro_rules! make{($meta:meta)=>{#[test]fn gone(){#[$meta]fn nested(){}}}}make!(cfg(ub_checks));
