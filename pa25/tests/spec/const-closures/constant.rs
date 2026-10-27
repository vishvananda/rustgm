#![feature(const_closures, const_trait_impl)]
const ANSWER: u8 = (const |value: u8| value)(7);
fn main() { assert_eq!(ANSWER, 7); }
