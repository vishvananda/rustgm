//! Ephemeral per-member source provenance; no persistent compiler permission.
use super::{ExpansionId, Output, SyntaxContextId};
use crate::{expand::builtin::Builtin, support::edition::Edition};
pub(crate) const MEMBERS: [Builtin; 10] = [
    Builtin::Copy,
    Builtin::Clone,
    Builtin::Debug,
    Builtin::Default,
    Builtin::Eq,
    Builtin::PartialEq,
    Builtin::Hash,
    Builtin::Ord,
    Builtin::PartialOrd,
    Builtin::ConstParamTy,
];
#[derive(Clone, Copy)]
pub(crate) struct MemberContext {
    pub context: (SyntaxContextId, ExpansionId),
    pub edition: Edition,
}
impl Output<'_> {
    pub(super) fn select(&mut self, slot: usize) -> bool {
        if let Some(member) = self.site.members[slot] {
            self.context = member.context;
            self.edition = member.edition;
            true
        } else {
            false
        }
    }
}
