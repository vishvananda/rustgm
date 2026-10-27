#![feature(rustc_attrs, allow_internal_unstable, const_trait_impl)]
#[allow_internal_unstable(const_closures)]
macro_rules! make { ($body:expr) => { const move |value: u8| $body }; }
const ANSWER: u8 = make!(7u8)(7);
fn main() {}
