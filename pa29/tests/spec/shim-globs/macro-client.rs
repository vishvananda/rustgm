#![no_std]
// The glob carries core's exported macro by path.
pub fn answer() -> u32 {
    core::forty_two!()
}
