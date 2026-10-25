#![feature(rustc_attrs)]
#![rustc_coherence_is_core]
impl &[u8] {
    pub fn __lib020_slice(self) -> u8 { 7 }
}
impl &[u16] {
    pub fn __lib020_slice(self) -> u16 { 9 }
}
pub fn byte_array_reference(value: &[u8; 3]) -> u8 { value.__lib020_slice() }
pub fn wide_array_reference(value: &[u16; 3]) -> u16 { value.__lib020_slice() }
pub fn byte_slice(value: &[u8]) -> u8 { value.__lib020_slice() }
pub fn wide_slice(value: &[u16]) -> u16 { value.__lib020_slice() }
