//! Exact comparison membership and provided bodies do not replace implementation proof.
use super::{DefId, Inventory};
use crate::{
    resolve::{model::BuiltinComparisonMembers, unit::NodeKey},
    support::comparisons::{Comparison, Method},
};
#[derive(Clone, Copy)]
pub(crate) enum LibraryDefault {
    PartialEqNe {
        eq: DefId,
    },
    OrdHelper {
        method: Method,
        cmp: DefId,
        partial: [DefId; 3],
    },
    PartialOrdRelation {
        partial_cmp: DefId,
        relation: Method,
    },
}
#[derive(Default)]
pub(super) struct Comparisons {
    ordering: Option<crate::types::ItemId>,
    owners: [Option<DefId>; 4],
    methods: [Option<BuiltinComparisonMembers>; 4],
}
impl Inventory {
    pub(in crate::sema) fn declare_comparison_ordering(
        &mut self,
        item: crate::types::ItemId,
    ) -> Result<(), &'static str> {
        let comparisons = self
            .language
            .get_or_insert_default()
            .comparisons
            .get_or_insert_default();
        if comparisons.ordering.is_some_and(|old| old != item) {
            return Err("comparison Ordering identity is already registered");
        }
        comparisons.ordering = Some(item);
        Ok(())
    }
    pub(crate) fn comparison_ordering(&self) -> Option<crate::types::ItemId> {
        self.language.as_deref()?.comparisons.as_deref()?.ordering
    }

    pub(in crate::sema) fn declare_comparison(
        &mut self,
        owner: DefId,
        kind: Comparison,
        origin: NodeKey,
        methods: Option<BuiltinComparisonMembers>,
    ) {
        let language = self.language.get_or_insert_default();
        let comparisons = language.comparisons.get_or_insert_default();
        comparisons.owners[kind as usize] = Some(owner);
        comparisons.methods[kind as usize] = methods;
        language.origins.insert(owner, origin);
    }
    pub(crate) fn comparison(&self, owner: DefId) -> Option<Comparison> {
        let comparisons = self.language.as_deref()?.comparisons.as_deref()?;
        Comparison::ALL
            .into_iter()
            .find(|kind| comparisons.owners[*kind as usize] == Some(owner))
    }
    pub(crate) fn comparison_method(&self, member: DefId) -> Option<Method> {
        self.language
            .as_deref()?
            .comparisons
            .as_deref()?
            .methods
            .iter()
            .flatten()
            .find_map(|methods| {
                methods
                    .iter()
                    .find_map(|(method, definition)| (definition == member).then_some(method))
            })
    }
    pub(crate) fn library_default(&self, member: DefId) -> Option<LibraryDefault> {
        self.library_defaults()
            .find_map(|(definition, default)| (definition == member).then_some(default))
    }
    pub(crate) fn library_defaults(&self) -> impl Iterator<Item = (DefId, LibraryDefault)> + '_ {
        self.language
            .as_deref()
            .and_then(|language| language.comparisons.as_deref())
            .into_iter()
            .flat_map(|comparisons| {
                comparisons
                    .methods
                    .iter()
                    .flatten()
                    .map(move |methods| (comparisons, methods))
            })
            .flat_map(|(comparisons, methods)| {
                methods.iter().filter_map(move |(method, definition)| {
                    let default = match method {
                        Method::Ne => LibraryDefault::PartialEqNe {
                            eq: methods.get(Method::Eq)?,
                        },
                        relation if relation.relation() => LibraryDefault::PartialOrdRelation {
                            partial_cmp: methods.get(Method::PartialCmp)?,
                            relation,
                        },
                        helper if helper.owned() => {
                            let partial = comparisons.methods[Comparison::PartialOrd as usize]?;
                            LibraryDefault::OrdHelper {
                                method: helper,
                                cmp: methods.get(Method::Cmp)?,
                                partial: [
                                    partial.get(Method::Le)?,
                                    partial.get(Method::Lt)?,
                                    partial.get(Method::Gt)?,
                                ],
                            }
                        }
                        _ => return None,
                    };
                    Some((definition, default))
                })
            })
    }
}
