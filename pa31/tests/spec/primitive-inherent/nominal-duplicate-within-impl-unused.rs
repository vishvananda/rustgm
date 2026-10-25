#![feature(rustc_attrs)]
#![rustc_coherence_is_core]
pub struct Local;
impl Local {
    fn __lib020_duplicate(self) -> u8 { 1 }
    fn __lib020_duplicate(self) -> u8 { 2 }
}
