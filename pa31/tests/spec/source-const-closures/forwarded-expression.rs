#![feature(rustc_attrs, allow_internal_unstable, const_trait_impl)]
#[allow_internal_unstable(const_closures)]
macro_rules! make { ($expression:expr) => { $expression }; }
const ANSWER: u8 = make!(const move |value: u8| value)(7);
fn main() {}
