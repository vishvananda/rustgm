// Exact builtin macro declaration intervals from pinned Rust88d9; module placement below is authored.
#[rustc_builtin_macro]
    pub macro derive($item:item) {
        /* compiler built-in */
    }
#[unstable(feature = "derive_const", issue = "118304")]
    #[rustc_builtin_macro]
    pub macro derive_const($item:item) {
        /* compiler built-in */
    }
#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics, derive_clone_copy_internals)]
pub macro Copy($item:item) {
    /* compiler built-in */
}
#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics, derive_clone_copy_internals, trivial_clone)]
pub macro Clone($item:item) {
    /* compiler built-in */
}
#[rustc_builtin_macro]
    #[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
    #[allow_internal_unstable(core_intrinsics, fmt_helpers_for_derive)]
    pub macro Debug($item:item) {
        /* compiler built-in */
    }
#[rustc_builtin_macro(Default, attributes(default))]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics)]
pub macro Default($item:item) {
    /* compiler built-in */
}


#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics, derive_eq_internals, structural_match)]
#[allow_internal_unstable(coverage_attribute)]
pub macro Eq($item:item) {
    /* compiler built-in */
}
#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics, structural_match)]
pub macro PartialEq($item:item) {
    /* compiler built-in */
}
#[rustc_builtin_macro]
    #[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
    #[allow_internal_unstable(core_intrinsics)]
    pub macro Hash($item:item) {
        /* compiler built-in */
    }
#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics)]
pub macro Ord($item:item) {
    /* compiler built-in */
}
#[rustc_builtin_macro]
#[stable(feature = "builtin_macro_prelude", since = "1.38.0")]
#[allow_internal_unstable(core_intrinsics)]
pub macro PartialOrd($item:item) {
    /* compiler built-in */
}
