//! Match direct methods by canonical trait member, retaining overrides separately from defaults.
use super::{Checker, DefId, Error, NodeKey};
use crate::support::hash::HashMap;
use crate::{
    resolve::{expansion::Report, model::DefKind},
    sema::{
        model::{Declarations, Function},
        traits::conformance,
    },
    syntax::item::{ItemKind, Safety, Visibility},
    types::parameters::predicates::Predicate,
};

#[derive(Default)]
pub(super) struct Members {
    owners: HashMap<DefId, Vec<DefId>>,
    libraries: HashMap<DefId, usize>,
}
impl Members {
    pub fn new(report: &Report, facts: &Declarations) -> Self {
        let mut result = Self::default();
        for (index, function) in facts.associated.language_functions().enumerate() {
            result.libraries.insert(function.definition, index);
        }
        for (member, owner) in facts.associated.declaration_members() {
            if report.resolution.definitions[member.0 as usize].kind == DefKind::Function {
                result.owners.entry(owner).or_default().push(member);
            }
        }
        result
    }

    fn function(&self, facts: &Declarations, definition: DefId) -> Result<Function, &'static str> {
        let function = if let Some(&index) = facts.function_indices.get(&definition) {
            facts.functions.get(index)
        } else {
            self.libraries
                .get(&definition)
                .and_then(|&index| facts.associated.language_function(index))
        }
        .ok_or("method inventory lost its actual function declaration")?;
        if function.definition != definition {
            return Err("method inventory index belongs to another actual declaration");
        }
        Ok(*function)
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
        let Some((id, _)) = predicate.bound.trait_parts() else {
            return Err(self.error(source, "missing impl trait predicate"));
        };
        let app = self
            .facts
            .types
            .trait_application(id)
            .ok_or_else(|| self.error(source, "missing impl trait application"))?;
        let trait_item = DefId(app.item.0);
        let trait_owner = self
            .facts
            .associated
            .declaration_owner(trait_item)
            .ok_or_else(|| self.error(source, "missing trait owner"))?;
        let impl_owner = self
            .facts
            .associated
            .declaration_owner(owner)
            .ok_or_else(|| self.error(source, "missing impl owner"))?;
        let overrides = super::super::override_identity::overrides(
            self.report,
            &self.facts.associated,
            trait_item,
            members.owners.get(&impl_owner).map_or(&[], Vec::as_slice),
            "implementation method is not a member of the trait",
        )
        .map_err(|(member, error)| {
            self.error(
                self.report.resolution.definitions[member.0 as usize]
                    .node
                    .unwrap_or(source),
                error,
            )
        })?;
        let mut result = Vec::new();
        for &definition in members.owners.get(&trait_owner).into_iter().flatten() {
            let expected = members
                .function(&self.facts, definition)
                .map_err(|e| self.error(source, e))?;
            if let Some(&actual) = overrides.get(&definition) {
                let actual = members
                    .function(&self.facts, actual)
                    .map_err(|e| self.error(source, e))?;
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
            } else if expected.body.is_some()
                || self
                    .facts
                    .associated
                    .library_default(expected.definition)
                    .is_some()
            {
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
