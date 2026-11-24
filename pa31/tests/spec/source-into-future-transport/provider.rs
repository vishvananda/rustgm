#[lang="freeze"]
pub unsafe auto trait Freeze {}

#[lang="future_trait"]
pub trait Future {
    #[lang="future_output"]
    type Output;
}

// Original pinned IntoFuture declaration.
