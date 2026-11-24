#![deny(unreachable_pub, unfulfilled_lint_expectations)]

#[expect(unreachable_pub)]
pub fn exported() -> u8 {
    17
}
