#![feature(rustc_attrs, allow_internal_unstable, const_trait_impl)]
#[allow_internal_unstable(const_closures)]
macro_rules! make { ($keyword:tt) => { $keyword move |value: u8| value }; }
const ANSWER: u8 = make!(const)(7);
fn main() {}
