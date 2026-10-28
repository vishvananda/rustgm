#![feature(no_core,rustc_attrs,prelude_import)]
#![no_core]
#[rustc_builtin_macro]
#[macro_export]
macro_rules! cfg { ($($t:tt)*) => { /* compiler built-in */ }; }
mod prelude { pub use crate::cfg; }
#[prelude_import] use crate::prelude::*;
#[no_implicit_prelude]
mod consumer { pub const CHECK: bool = cfg!(unix); }
