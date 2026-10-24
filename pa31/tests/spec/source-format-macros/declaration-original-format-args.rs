#![feature(no_core,lang_items,rustc_attrs,allow_internal_unsafe,allow_internal_unstable,staged_api)]
#![no_core]
#![stable(feature="format_fixture",since="1.0.0")]
#[stable(feature="format_fixture",since="1.0.0")]
#[lang="pointee_sized"] pub trait PointeeSized {}
#[stable(feature="format_fixture",since="1.0.0")]
#[lang="meta_sized"] pub trait MetaSized:PointeeSized {}
#[stable(feature="format_fixture",since="1.0.0")]
#[lang="sized"] pub trait Sized:MetaSized {}
    #[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_diagnostic_item = "format_args_macro"]
    #[allow_internal_unsafe]
    #[allow_internal_unstable(fmt_internals, fmt_arguments_from_str)]
    #[rustc_builtin_macro]
    #[macro_export]
    macro_rules! format_args {
        ($fmt:expr) => {{ /* compiler built-in */ }};
        ($fmt:expr, $($args:tt)*) => {{ /* compiler built-in */ }};
    }
#[stable(feature="format_fixture",since="1.0.0")]
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { 7u32 }
