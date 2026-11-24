#![feature(diagnostic_on_const, diagnostic_on_unmatched_args)]
#![deny(malformed_diagnostic_attributes, malformed_diagnostic_format_literals)]
#![deny(misplaced_diagnostic_attributes)]

pub trait Lookup {}

#[diagnostic::on_const(message = "could not evaluate {Self}", note = "use a runtime call")]
impl Lookup for u8 {}

#[cfg_attr(not(empty), diagnostic::on_unmatched_args(note = "call {This} with no arguments"))]
#[cfg_attr(empty, diagnostic::on_unmatched_args())]
macro_rules! answer {
    () => { 34u8 };
}

pub fn check(value: impl Lookup) -> u8 {
    let _ = value;
    answer!()
}

#[cfg(misplaced)]
#[diagnostic::on_const(note = "a trait implementation is required")]
pub fn wrong_owner() {}

#[cfg(format)]
#[diagnostic::on_unmatched_args(note = "unknown parameter {Generic}")]
macro_rules! wrong_format {
    () => { 5u8 };
}

#[cfg(format)]
pub fn uses_wrong_format() -> u8 {
    wrong_format!()
}
