#![feature(rustc_attrs)]
#![feature(staged_api)]
#![feature(no_core)]
#![no_core]
#![stable(feature="rust1", since="1.0.0")]
#![unstable_removed(
    feature = "concat_idents",
    reason = "Replaced by the macro_metavar_expr_concat feature",
    link = "https://github.com/rust-lang/rust/issues/29599#issuecomment-2986866250",
    since = "1.90.0"
)]
#[stable(feature="rust1", since="1.0.0")]
pub mod api {
#[stable(feature="rust1", since="1.0.0")]
pub mod child {}
}
