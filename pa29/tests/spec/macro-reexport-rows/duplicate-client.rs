#![feature(no_core)]
#![no_core]
extern crate exporter;
use exporter::line;
use exporter::prelude::v1::line;
pub fn twice() -> u32 {
    line!()
}
