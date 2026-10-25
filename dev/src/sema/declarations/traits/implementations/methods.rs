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
    pub(super) fn prepare_source_owner(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<(), &'static str> {
        if self.owners.contains_key(&owner) {
            return Ok(());
        }
        let id = facts
            .associated
            .owner(owner)
            .ok_or("source method owner is missing")?;
        let actual = facts
            .associated
            .get(id)
            .ok_or("source method owner disappeared")?;
        if actual.self_definition != owner
            || !matches!(
                actual.kind,
                crate::sema::associated::OwnerKind::Impl { .. }
                    | crate::sema::associated::OwnerKind::Trait { .. }
            )
        {
            return Err("early method inventory requires an actual source owner");
        }
        actual.source_owner()?;
        let members = facts
            .associated
            .owner_members(id)
            .ok_or("source method membership is missing")?;
        let mut functions = Vec::new();
        for &member in members {
            if facts.associated.owner(member) != Some(id) {
                return Err("source method membership belongs to another owner");
            }
            if report.resolution.definitions[member.0 as usize].kind == DefKind::Function {
                functions.push(member);
            }
        }
        self.owners.insert(owner, functions);
        Ok(())
    }

    fn function(
        &self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<Option<Function>, &'static str> {
        let function = if let Some(&index) = facts.function_indices.get(&definition) {
            Some(
                facts
                    .functions
                    .get(index)
                    .ok_or("method inventory has a stale source index")?,
            )
        } else if let Some(&index) = self.libraries.get(&definition) {
            Some(
                facts
                    .associated
                    .language_function(index)
                    .ok_or("method inventory has a stale library index")?,
            )
        } else {
            None
        };
        if function.is_some_and(|function| function.definition != definition) {
            return Err("method inventory index belongs to another actual declaration");
        }
        Ok(function.copied())
    }
    fn expected(
        &self,
        facts: &mut Declarations,
        definition: DefId,
    ) -> Result<(conformance::Method, bool), &'static str> {
        if let Some(function) = self.function(facts, definition)? {
            return Ok((
                conformance::Method::source(facts, function),
                function.body.is_some(),
            ));
        }
        let method = conformance::Method::imported(facts, definition)?;
        let body = facts
            .function_header(definition)?
            .ok_or("imported conformance lost its required method header")?
            .body
            .is_some();
        Ok((method, body))
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
            let (expected, has_body) = members
                .expected(&mut self.facts, definition)
                .map_err(|e| self.error(source, e))?;
            if let Some(&actual) = overrides.get(&definition) {
                let actual = members
                    .function(&self.facts, actual)
                    .map_err(|e| self.error(source, e))?
                    .ok_or_else(|| {
                        self.error(source, "source override lacks its actual function syntax")
                    })?;
                self.trait_override_contract(owner, predicate, expected, actual, definitions)?;
                result.push((expected.definition, Some(actual.definition)));
            } else if has_body
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
    fn trait_override_contract(
        &mut self,
        owner: DefId,
        predicate: Predicate,
        expected: conformance::Method,
        actual: Function,
        definitions: &conformance::Definitions,
    ) -> Result<(), Error> {
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
        let actual_method = conformance::Method::source(&self.facts, actual);
        conformance::check_methods(
            &mut self.facts,
            owner,
            predicate,
            expected,
            actual_method,
            definitions,
        )
        .map_err(|e| self.error(actual.node, e))
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
