struct Condition; impl std::ops::Not for Condition { type Output = u8; fn not(self) -> u8 { 0 } } fn main() { assert!(Condition); }
