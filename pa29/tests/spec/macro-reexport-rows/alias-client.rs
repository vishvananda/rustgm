#![feature(no_core)]
#![no_core]
extern crate exporter as renamed;
// A local module with the provider's prelude name binds nothing there.
pub mod prelude {}
use renamed::prelude::v1::forty_two;
pub fn spelled() -> u32 {
    let _answer: u32 = forty_two!();
    let _joined: &str = renamed::prelude::rust_2024::concat!("x", 2);
    let _spelled: &str = renamed::spelled!(y);
    renamed::prelude::v1::line!()
}
