#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[macro_export]
#[rustc_builtin_macro]
macro_rules! format_args {
    ($($tokens:tt)*) => {{}};
}

#[macro_export]
#[rustc_builtin_macro]
macro_rules! format_args_nl {
    ($($tokens:tt)*) => {{}};
}

#[macro_export]
#[rustc_builtin_macro]
macro_rules! const_format_args {
    ($($tokens:tt)*) => {{}};
}

#[macro_export]
#[allow_internal_unstable(stmt_expr_attributes)]
macro_rules! selected {
    () => {
        #[allow(unused)]
        true
    };
}

#[macro_export]
#[allow_internal_unstable(stmt_expr_attributes)]
macro_rules! forward {
    ($value:expr) => { $value };
}

#[lang = "format_arguments"]
pub struct TextArguments<'a> {
    pub text: &'a str,
}

impl<'a> TextArguments<'a> {
    pub const fn from_str(text: &'static str) -> Self {
        Self { text }
    }
}
