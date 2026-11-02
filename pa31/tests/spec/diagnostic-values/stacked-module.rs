#![feature(no_core,rustc_attrs)]
#![no_core]
#![allow(internal_features,unused_macros,dead_code)]
#[rustc_builtin_macro]
macro_rules! concat { () => {} }
#[rustc_builtin_macro]
macro_rules! stringify { () => {} }
macro_rules! legacy_int_module {
    ($T:ident) => {legacy_int_module!($T, #[allow(dead_code)]);};
    ($T:ident, #[$attr:meta]) => {
        #[$attr]
        #[deprecated(since="1.0.0",note="legacy")]
        #[rustc_diagnostic_item=concat!(stringify!($T),"_legacy_mod")]
        pub mod $T {
            #![doc=concat!("Legacy ",stringify!($T))]
        }
    };
}
legacy_int_module!(u8);legacy_int_module!(i16);
