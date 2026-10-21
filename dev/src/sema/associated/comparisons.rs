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
    PartialOrdRelation {
        partial_cmp: DefId,
        relation: Method,
    },
}
#[derive(Default)]
pub(super) struct Comparisons {
    owners: [Option<DefId>; 4],
    methods: [Option<BuiltinComparisonMembers>; 4],
}
impl Inventory {
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
            .flat_map(|comparisons| comparisons.methods.iter().flatten())
            .flat_map(|methods| {
                methods.iter().filter_map(move |(method, definition)| {
                    let default = match method {
                        Method::Ne => LibraryDefault::PartialEqNe {
                            eq: methods.get(Method::Eq)?,
                        },
                        relation if relation.relation() => LibraryDefault::PartialOrdRelation {
                            partial_cmp: methods.get(Method::PartialCmp)?,
                            relation,
                        },
                        _ => return None,
                    };
                    Some((definition, default))
                })
            })
    }
}
