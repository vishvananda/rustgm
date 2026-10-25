struct Condition(bool); impl std::ops::Not for Condition { type Output = bool; fn not(self) -> bool { !self.0 } } fn main() { assert!(Condition(true)); println!("ok"); }
