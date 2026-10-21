use core::sync::atomic::AtomicU32;
unsafe extern "Rust" {
    #[link_name = "foreign_atomic"]
    safe static SHARED: AtomicU32;
    #[link_name = "foreign_ordinary"]
    safe static ORDINARY: u32;
}
fn main() {
    let _ = &SHARED;
    let _ = &ORDINARY;
}
