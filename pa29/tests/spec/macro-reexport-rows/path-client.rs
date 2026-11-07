#![feature(no_core)]
#![no_core]
extern crate exporter;
use exporter::prelude::v1::column;
#[unsafe(no_mangle)]
pub extern "C" fn invoked_line() -> u32 {
    exporter::prelude::rust_2024::line!()
}
#[unsafe(no_mangle)]
pub extern "C" fn invoked_column() -> u32 {
    let _joined: &str = exporter::builtin::concat!("a", 1);
    let _file: &str = exporter::file!();
    let _spelled: &str = exporter::spelled!(b c);
    let _answer: u32 = exporter::prelude::v1::forty_two!();
        column!()
}
#[unsafe(no_mangle)]
pub extern "C" fn configured() -> bool {
    exporter::prelude::rust_2024::cfg!(target_os = "linux")
}
