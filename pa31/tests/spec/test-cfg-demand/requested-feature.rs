#![allow(dead_code,unused_imports,unused_macros,internal_features)]
#![feature(cfg_ub_checks)]
#[test]fn gone(){#[cfg(ub_checks)]fn nested(){}}
