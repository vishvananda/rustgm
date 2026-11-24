#![feature(doc_cfg)]
#![deny(invalid_doc_attributes, unfulfilled_lint_expectations)]

#[cfg_attr(not(lint_denied), expect(invalid_doc_attributes))]
#[doc(auto_cfg(unknown(x)))]
pub fn active() -> u8 {
    23
}
