mod implementation {
    pub struct Handle { pub value: u32, secret: u32 }
    pub fn make() -> Handle { Handle { value: 9, secret: 1 } }
    pub fn value(item: Handle) -> u32 { item.value + item.secret - 1 }
    pub static DATA: u32 = 9;
    pub const NUMBER: u32 = 9;
}
pub use implementation::{Handle, make, value, DATA, NUMBER};
pub mod surface {
    pub use super::implementation::{make, DATA};
    pub use super::surface as again;
}
pub mod empty {}
