#![feature(no_core)]
#![no_core]
extern crate apis;
pub fn seven() -> u32 {
    apis::leaky_api!()
}
