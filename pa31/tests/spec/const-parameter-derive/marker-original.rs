#[lang = "const_param_ty"]
#[unstable(feature = "const_param_ty_trait", issue = "95174", implied_by = "unsized_const_params")]
#[diagnostic::on_unimplemented(message = "`{Self}` can't be used as a const parameter type")]
#[allow(multiple_supertrait_upcastable)]
// We name this differently than the derive macro so that the `adt_const_params` can
// be used independently of `unsized_const_params` without requiring a full path
// to the derive macro every time it is used. This should be renamed on stabilization.
pub trait ConstParamTy_: StructuralPartialEq + Eq {}

/// Derive macro generating an impl of the trait `ConstParamTy`.
#[rustc_builtin_macro]
#[allow_internal_unstable(const_param_ty_trait)]
#[unstable(feature = "min_adt_const_params", issue = "154042", implied_by = "adt_const_params")]
pub macro ConstParamTy($item:item) {
    /* compiler built-in */
}

