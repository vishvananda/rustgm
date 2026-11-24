//! Documentation for the crate.
#![deny(missing_docs, unfulfilled_lint_expectations)]

mod details {
    /// Documented through a public re-export.
    pub fn documented() -> u8 {
        9
    }

    #[cfg_attr(not(lint_denied), expect(missing_docs))]
    pub struct Exposed {
        /// Named field.
        pub value: u8,
    }

    #[doc(hidden)]
    pub struct Hidden;

    impl Hidden {
        pub fn omitted(&self) {}
    }
}

pub use details::{Exposed, Hidden, documented};
