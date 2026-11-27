#[unstable(feature = "lexical_function", issue = "none")]
pub fn exposed() -> u8 {
    const VALUE: u8 = 29;
    fn hidden(value: u8) -> u8 {
        <Hidden as Internal>::keep(value)
    }
    hidden(VALUE)
}

#[stable(feature = "stable_constructor", since = "1.0.0")]
pub struct Mark(#[stable(feature = "stable_constructor", since = "1.0.0")] pub u8);

#[unstable(feature = "anonymous_policy", issue = "none")]
const _: () = ();

#[stable(feature = "stable_variants", since = "1.0.0")]
#[repr(u8)]
pub enum Marker {
    First = 41,
    Second = 72,
}

#[stable(feature = "const_policy_trait", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_policy_runtime", issue = "none")]
pub const trait Evaluation {
    #[stable(feature = "const_policy_trait", since = "1.0.0")]
    #[rustc_non_const_trait_method]
    fn runtime() -> u8 {
        63
    }
}

#[stable(feature = "const_policy_trait", since = "1.0.0")]
#[rustc_const_unstable(feature = "const_policy_runtime", issue = "none")]
impl Evaluation for Mark {}

#[unstable(feature = "lexical_function", issue = "none")]
pub const EXPOSED: u8 = {
    const INNER: u8 = 37;
    INNER
};

trait Internal {
    fn keep(value: u8) -> u8;
}

struct Hidden;

impl Internal for Hidden {
    fn keep(value: u8) -> u8 {
        value
    }
}

#[stable(feature = "availability_panic", since = "1.0.0")]
#[rustc_const_stable(feature = "availability_panic", since = "1.0.0")]
#[lang = "panic_const_add_overflow"]
pub const fn panic_const_add_overflow() -> ! {
    loop {}
}

#[stable(feature = "availability_derive", since = "1.0.0")]
pub mod clone {
    #[stable(feature = "availability_derive", since = "1.0.0")]
    pub use crate::Clone;
}

#[rustc_builtin_macro]
#[stable(feature = "availability_derive", since = "1.0.0")]
pub macro derive($item:item) {}

#[rustc_builtin_macro]
#[stable(feature = "availability_derive", since = "1.0.0")]
pub macro Clone($item:item) {}

#[derive(Clone)]
#[stable(feature = "availability_derive", since = "1.0.0")]
pub struct Derived;

impl Derived {
    #[stable(feature = "availability_derive", since = "1.0.0")]
    pub fn value(&self) -> u8 {
        91
    }
}

#[stable(feature = "availability_roles", since = "1.0.0")]
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
