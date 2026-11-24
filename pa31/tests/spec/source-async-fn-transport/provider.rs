#[lang="freeze"]
pub unsafe auto trait Freeze {}

#[lang="future_trait"]
pub trait Future {
    #[lang="future_output"]
    type Output;
}

#[lang="tuple_trait"]
#[rustc_deny_explicit_impl]
pub trait Tuple {}

// Original pinned async callable declarations.
