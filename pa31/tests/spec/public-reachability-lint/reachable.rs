#![deny(unreachable_pub, unfulfilled_lint_expectations)]
#![allow(dead_code)]

mod details {
    pub fn exposed() -> u8 {
        7
    }

    pub struct InSignature;

    impl InSignature {
        pub fn method(self) -> u8 {
            23
        }
    }

    #[expect(unreachable_pub)]
    pub fn hidden() {}
}

pub use details::exposed;

pub fn signature() -> details::InSignature {
    details::InSignature
}
