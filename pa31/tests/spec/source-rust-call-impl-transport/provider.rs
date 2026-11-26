#[lang = "freeze"]
pub unsafe auto trait Freeze {}

pub struct Identity;

impl FnOnce<(u8,)> for Identity {
    type Output = u8;

    extern "rust-call" fn call_once(self, (value,): (u8,)) -> u8 {
        value
    }
}
