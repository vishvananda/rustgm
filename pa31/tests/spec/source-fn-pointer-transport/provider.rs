#[lang="freeze"] pub unsafe auto trait Freeze {}

// Original declaration from original-marker.rs

pub trait Select { fn selected(&self) -> u8; }
impl<F: FnPtr> Select for F { fn selected(&self) -> u8 { 17 } }
impl<T> Select for &T { fn selected(&self) -> u8 { 29 } }
