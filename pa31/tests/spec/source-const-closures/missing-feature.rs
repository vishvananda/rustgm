#![feature(const_trait_impl)]
const ANSWER: u8 = (const move |value: u8| value)(7);
fn main() {}
