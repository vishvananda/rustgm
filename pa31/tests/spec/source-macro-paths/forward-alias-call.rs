#![feature(no_core)]
#![no_core]
#![allow(internal_features, unused_imports, unused_macros)]
mod prelude { pub use crate::option_env as selected; selected!(); }
#[macro_export] macro_rules! option_env { () => {}; }
