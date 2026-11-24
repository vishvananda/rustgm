#[lang = "structural_peq"]
pub trait Structural {}

#[rustc_diagnostic_item = "Eq"]
pub trait Equality {}

#[lang = "const_param_ty"]
pub trait Key: Structural + Equality {}

#[rustc_diagnostic_item = "Send"]
pub unsafe auto trait Other {}

impl Structural for usize {}
impl Equality for usize {}
impl Key for usize {}

pub fn marker() -> u8 {
    37
}

pub fn scalar<const VALUE: usize>() -> usize {
    VALUE
}
