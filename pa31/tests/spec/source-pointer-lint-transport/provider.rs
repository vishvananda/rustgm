#[cfg(rustc_oracle)]
#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[must_use = "keep the returned pointer"]
#[rustc_as_ptr]
pub fn pointer(value: *const u8) -> *const u8 {
    value
}

pub fn plain(value: u8) -> u8 {
    value
}

#[cfg(not(older_profile))]
pub struct Pointer {
    pub value: u8,
}

#[cfg(not(older_profile))]
impl Pointer {
    #[rustc_as_ptr]
    pub fn as_ptr(&self) -> *const u8 {
        &self.value as *const u8
    }
}
