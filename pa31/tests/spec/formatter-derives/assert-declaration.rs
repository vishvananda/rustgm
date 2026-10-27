// Exact declaration interval from pinned Rust88d9 core/macros/mod.rs.
#[stable(feature = "rust1", since = "1.0.0")]
    #[rustc_builtin_macro]
    #[macro_export]
    #[rustc_diagnostic_item = "assert_macro"]
    #[allow_internal_unstable(
        core_intrinsics,
        panic_internals,
        edition_panic,
        generic_assert_internals
    )]
    macro_rules! assert {
        ($cond:expr $(,)?) => {{ /* compiler built-in */ }};
        ($cond:expr, $($arg:tt)+) => {{ /* compiler built-in */ }};
    }
