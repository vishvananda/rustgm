#![feature(rustc_attrs)]
#![rustc_coherence_is_core]
impl u8 {
    fn __lib020_duplicate(self) -> u8 { 1 }
    fn __lib020_duplicate(self) -> u8 { 2 }
    pub fn __lib020_caller(self) -> u8 { Self::__lib020_duplicate(self) }
}
