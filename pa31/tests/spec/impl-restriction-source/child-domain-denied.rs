#![feature(impl_restriction)]
pub mod owner { pub mod child {} pub impl(in self::child) trait Restricted {} }
