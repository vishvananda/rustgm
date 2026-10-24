#![feature(no_core,lang_items)]
#![no_core]
#![feature(decl_macro,rustc_attrs,allow_internal_unstable,staged_api)]
#![stable(feature="declaration_fixture",since="1.0.0")]
// Declaration below is byte-exact from pinned Rust core; attribution is in declaration-contract.json.
#[stable(feature = "assert_matches", since = "1.96.0")]
#[allow_internal_unstable(panic_internals)]
#[rustc_macro_transparency = "semiopaque"]
pub macro assert_matches {
    ($left:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {{
        match $left {
            $( $pattern )|+ $( if $guard )? => {}
            ref left_val => {
                $crate::panicking::assert_matches_failed(
                    left_val,
                    $crate::stringify!($($pattern)|+ $(if $guard)?),
                    $crate::option::Option::None
                );
            }
        }
    }},
    ($left:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?, $($arg:tt)+) => {{
        match $left {
            $( $pattern )|+ $( if $guard )? => {}
            ref left_val => {
                $crate::panicking::assert_matches_failed(
                    left_val,
                    $crate::stringify!($($pattern)|+ $(if $guard)?),
                    $crate::option::Option::Some($crate::format_args!($($arg)+))
                );
            }
        }
    }},
}
#[stable(feature="declaration_fixture",since="1.0.0")]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[stable(feature="declaration_fixture",since="1.0.0")]
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[stable(feature="declaration_fixture",since="1.0.0")]
#[lang="sized"]pub trait Sized:MetaSized{}
#[stable(feature="declaration_fixture",since="1.0.0")]
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32{7u32}
