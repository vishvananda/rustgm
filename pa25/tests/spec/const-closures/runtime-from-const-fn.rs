#![feature(const_closures, const_trait_impl)]
const fn make() -> impl Fn(u8) -> u8 { const move |value: u8| value }
fn main() { assert_eq!(make()(7), 7); }
