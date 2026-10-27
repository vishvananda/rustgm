//! Compound operator tokens select their canonical language-hook vocabulary.
use crate::{lex::token::Punctuation, support::language_hooks::Hook};
pub(crate) const fn hook(operator: Punctuation) -> Option<Hook> {
    match operator {
        Punctuation::PlusEqual => Some(Hook::AddAssign),
        Punctuation::MinusEqual => Some(Hook::SubAssign),
        Punctuation::StarEqual => Some(Hook::MulAssign),
        Punctuation::SlashEqual => Some(Hook::DivAssign),
        Punctuation::PercentEqual => Some(Hook::RemAssign),
        Punctuation::AndEqual => Some(Hook::BitAndAssign),
        Punctuation::OrEqual => Some(Hook::BitOrAssign),
        Punctuation::CaretEqual => Some(Hook::BitXorAssign),
        Punctuation::ShiftLeftEqual => Some(Hook::ShlAssign),
        Punctuation::ShiftRightEqual => Some(Hook::ShrAssign),
        _ => None,
    }
}
