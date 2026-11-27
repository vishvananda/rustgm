#[lang = "coerce_pointee_validated"]
pub trait CoercePointeeValidated {}

pub fn use_marker<T: CoercePointeeValidated>(value: T) -> T {
    value
}

pub fn identity(value: u8) -> u8 {
    value
}
