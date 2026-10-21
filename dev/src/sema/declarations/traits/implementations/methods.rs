//! Match direct methods once by owner/name, retaining overrides separately from defaults.
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    resolve::{expansion::Report, names::NameId},
    sema::{
        model::{Declarations, Function},
        traits::conformance,
    },
    syntax::item::{ItemKind, Safety, Visibility},
    types::parameters::predicates::{Bound, Predicate},
};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Members {
    owners: HashMap<DefId, Vec<Function>>,
    names: HashMap<(DefId, NameId), Function>,
}
impl Members {
    pub fn new(report: &Report, facts: &Declarations) -> Self {
        let mut result = Self::default();
        for &function in facts
            .functions
            .iter()
            .chain(facts.associated.language_functions())
        {
            if let Some(owner) = facts.associated.member_owner(function.definition) {
                let name = report.resolution.definitions[function.definition.0 as usize].name;
                result.names.insert((owner, name), function);
            }
        }
        for (member, owner) in facts.associated.declaration_members() {
            let name = report.resolution.definitions[member.0 as usize].name;
            if let Some(&function) = result.names.get(&(owner, name))
                && function.definition == member
            {
                result.owners.entry(owner).or_default().push(function);
            }
        }
        result
    }
}
impl Checker<'_> {
    pub(super) fn trait_impl_methods(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        members: &Members,
        definitions: &conformance::Definitions,
    ) -> Result<Vec<(DefId, Option<DefId>)>, Error> {
        let Bound::Trait(id) = predicate.bound else {
            return Err(self.error(source, "missing impl trait predicate"));
        };
        let app = self
            .facts
            .types
            .trait_application(id)
            .ok_or_else(|| self.error(source, "missing impl trait application"))?;
        let trait_owner = self
            .facts
            .associated
            .declaration_owner(DefId(app.item.0))
            .ok_or_else(|| self.error(source, "missing trait owner"))?;
        let impl_owner = self
            .facts
            .associated
            .declaration_owner(owner)
            .ok_or_else(|| self.error(source, "missing impl owner"))?;
        for function in members.owners.get(&impl_owner).into_iter().flatten() {
            let name = self.report.resolution.definitions[function.definition.0 as usize].name;
            if !members.names.contains_key(&(trait_owner, name)) {
                return Err(self.error(
                    function.node,
                    "implementation method is not a member of the trait",
                ));
            }
        }
        let mut result = Vec::new();
        for &expected in members.owners.get(&trait_owner).into_iter().flatten() {
            let name = self.report.resolution.definitions[expected.definition.0 as usize].name;
            if let Some(&actual) = members.names.get(&(impl_owner, name)) {
                self.trait_override(actual)?;
                if self
                    .facts
                    .methods
                    .receivers
                    .contains_key(&expected.definition)
                    != self
                        .facts
                        .methods
                        .receivers
                        .contains_key(&actual.definition)
                {
                    return Err(self.error(
                        actual.node,
                        "implementation method receiver presence disagrees with trait",
                    ));
                }
                conformance::check(
                    &mut self.facts,
                    owner,
                    predicate,
                    expected,
                    actual,
                    definitions,
                )
                .map_err(|e| self.error(actual.node, e))?;
                result.push((expected.definition, Some(actual.definition)));
            } else if expected.body.is_some() {
                result.push((expected.definition, None));
            } else {
                return Err(self.error(source, "implementation is missing a required trait method"));
            }
        }
        Ok(result)
    }
    fn trait_override(&self, function: Function) -> Result<(), Error> {
        let item = self.item(function.node)?;
        let ItemKind::Function { flags, abi, .. } = item.kind else {
            return Err(self.error(function.node, "missing implementation method syntax"));
        };
        if item.visibility != Visibility::Private
            || flags.safety == Safety::Safe
            || flags.constant
            || flags.asynchronous
            || !crate::sema::declarations::rust_abi(self.report, function.node, abi)?
            || function.body.is_none()
        {
            return Err(self.error(function.node,"trait implementation methods require inherited visibility, a Rust body, and supported function qualifiers"));
        }
        Ok(())
    }
}
