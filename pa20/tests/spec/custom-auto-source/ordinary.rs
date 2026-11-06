#![feature(auto_traits)]
auto trait Auto {}
struct Record(u8);
fn need<T: Auto>(value: T) -> T { value }
pub fn check(value: u8) -> u8 { need(Record(value)).0 }
