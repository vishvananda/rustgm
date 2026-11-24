#![feature(staged_api)]
#![stable(feature = "base", since = "1.0.0")]
#![allow(internal_features, dead_code)]
#![deny(unfulfilled_lint_expectations)]

#[stable(feature = "base", since = "1.0.0")]
trait Visit<'a> {}

#[stable(feature = "base", since = "1.0.0")]
struct Borrowed<'a, 'b>(&'a u8, &'b u8);

#[cfg_attr(not(lint_denied), expect(ineffective_unstable_trait_impl))]
#[unstable(feature = "sample", issue = "none")]
impl<'a> Visit<'a> for Borrowed<'a, '_> {}
