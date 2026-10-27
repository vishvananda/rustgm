#![feature(const_closures)]
fn main() { let f = const move |value: u8| value; assert_eq!(f(7), 7); }
