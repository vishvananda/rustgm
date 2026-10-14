mod m {
    pub struct S;
    pub trait A {
        fn get(&self) -> u8 { 7 }
    }
    impl A for S {}
}
mod exported { pub use crate::m::A as B; }
fn main() { let _ = exported::B::get(&m::S); }
