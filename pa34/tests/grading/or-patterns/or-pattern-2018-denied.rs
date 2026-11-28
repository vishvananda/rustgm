#![allow(unused_macros)]
#![deny(rust_2021_incompatible_or_patterns, unfulfilled_lint_expectations)]
#![cfg_attr(lint_expected, expect(rust_2021_incompatible_or_patterns))]

#[cfg(any(lint_expected, lint_denied))]
macro_rules! legacy {
    ($first:pat | $second:pat) => { 12u8 };
}

macro_rules! migrated {
    ($first:pat_param | $second:pat_param) => { 26u8 };
}

pub fn result() -> u8 {
    migrated!(0 | 1)
}
