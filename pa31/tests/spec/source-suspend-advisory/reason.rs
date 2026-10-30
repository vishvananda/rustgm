#![feature(must_not_suspend)]
#[must_not_suspend="holding a guard may cause a BorrowError"]pub struct S(pub u32);
