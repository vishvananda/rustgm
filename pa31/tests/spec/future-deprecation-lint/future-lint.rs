#![deny(deprecated, unfulfilled_lint_expectations)]

#[deprecated(since = "TBD", note = "use new instead")]
fn old() -> u8 {
    14
}

#[cfg(not(lint_denied))]
#[deprecated(since = "TBD", note = "use new instead")]
macro_rules! old_macro {
    () => { 27u8 };
}

#[cfg(lint_denied)]
#[deny(deprecated_in_future)]
pub fn use_old() -> u8 {
    old()
}

#[cfg(not(lint_denied))]
#[expect(deprecated_in_future)]
pub fn use_old() -> u8 {
    old() + old_macro!()
}
