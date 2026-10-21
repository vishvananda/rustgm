//! Exact comparison membership and provided bodies do not replace implementation proof.
use super::{DefId, Inventory};
use crate::{
    resolve::unit::NodeKey,
    support::comparisons::{Comparison, Method},
};
#[derive(Clone, Copy)]
pub(crate) enum LibraryDefault {
    PartialEqNe { eq: DefId },
}
#[derive(Default)]
pub(super) struct Comparisons {
    owners: [Option<DefId>; 2],
    methods: Option<[DefId; 2]>,
}
impl Inventory {
    pub(in crate::sema) fn declare_comparison(
        &mut self,
        owner: DefId,
        kind: Comparison,
        origin: NodeKey,
        methods: Option<[DefId; 2]>,
    ) {
        let language = self.language.get_or_insert_default();
        let comparisons = language.comparisons.get_or_insert_default();
        comparisons.owners[kind as usize] = Some(owner);
        if methods.is_some() {
            comparisons.methods = methods;
        }
        language.origins.insert(owner, origin);
    }
    pub(crate) fn comparison(&self, owner: DefId) -> Option<Comparison> {
        let comparisons = self.language.as_deref()?.comparisons.as_deref()?;
        Comparison::ALL
            .into_iter()
            .find(|kind| comparisons.owners[*kind as usize] == Some(owner))
    }
    pub(crate) fn comparison_method(&self, member: DefId) -> Option<Method> {
        let methods = self.language.as_deref()?.comparisons.as_deref()?.methods?;
        Method::ALL
            .into_iter()
            .find(|method| methods[*method as usize] == member)
    }
    pub(crate) fn library_default(&self, member: DefId) -> Option<LibraryDefault> {
        self.library_defaults()
            .find_map(|(definition, default)| (definition == member).then_some(default))
    }
    pub(crate) fn library_defaults(&self) -> impl Iterator<Item = (DefId, LibraryDefault)> + '_ {
        self.language
            .as_deref()
            .and_then(|language| language.comparisons.as_deref())
            .and_then(|comparisons| comparisons.methods)
            .into_iter()
            .map(|methods| {
                (
                    methods[Method::Ne as usize],
                    LibraryDefault::PartialEqNe {
                        eq: methods[Method::Eq as usize],
                    },
                )
            })
    }
}
