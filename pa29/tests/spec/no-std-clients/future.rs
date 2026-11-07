#![no_std]
// Only the Rust 2024 prelude carries `IntoFuture`.
struct Ready;
impl IntoFuture for Ready {}
