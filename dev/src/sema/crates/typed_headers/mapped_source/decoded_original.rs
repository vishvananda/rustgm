//! Sealed source data binds one actual original to the installed canonical graph.
use super::{Catalog, DefId, DefKind, Graph, s, slot_bindings};
use crate::{
    resolve::body_origin::{BodyOrigin, BodySite, ImportedBodyId, ProofSite},
    sema::model::Declarations,
    support::crates::{generics::TypeRef, owner_catalog::DeclarationRef},
    types::{
        RegionId, TypeId,
        arguments::Argument,
        parameters::{ParameterId, predicates::Predicate},
    },
};
use std::sync::Arc;

/// Borrowed reconstruction inputs, not a checked body, permission or executable.
/// Only the actual current Graph/header/catalog correspondence can construct one.
pub(crate) struct Original<'a> {
    graph: &'a Graph,
    catalog: &'a Catalog,
    header: &'a super::super::Header,
    definition: DefId,
    body: ImportedBodyId,
    rows: s::ValidatedView<'a>,
    reference: crate::support::crates::owner_catalog::BodyRef,
}
impl Graph {
    pub(in crate::sema::crates::typed_headers) fn owns_original(
        &self,
        catalog: &Catalog,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<bool, &'static str> {
        charge(steps)?;
        let descriptor = self.descriptor(catalog)?;
        let declaration = catalog.declaration(definition)?;
        catalog.validate_declaration(declaration, definition, DefKind::Function)?;
        let (owner, _, _) = catalog.declaration_coordinate(declaration)?;
        Ok(owner == descriptor.identity())
    }
    pub(in crate::sema::crates::typed_headers) fn source_original<'a>(
        &'a self,
        catalog: &'a Catalog,
        facts: &Declarations,
        headers: &'a super::super::Headers,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<Option<Original<'a>>, &'static str> {
        self.function_ready(catalog, facts, headers, definition, steps)?;
        let header = headers
            .get(definition)
            .ok_or("source original header disappeared")?;
        let Some(origin) = header.body else {
            return Ok(None);
        };
        let descriptor = self.descriptor(catalog)?;
        let declaration = catalog.declaration(definition)?;
        let (_, row, _) = catalog.declaration_coordinate(declaration)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Function(index)) = rows.header(row)? else {
            return Err("source original lost its actual function/header association");
        };
        let function = &rows.functions[index as usize];
        let s::FunctionBody::Original(reference) = function.body else {
            return Err("source original cannot reconstruct an absent required method");
        };
        let body = catalog.body_at(descriptor.identity(), reference)?;
        let raw = rows
            .originals
            .get(reference.0 as usize)
            .filter(|raw| raw.body == reference && raw.declaration == row)
            .ok_or("source provided body omitted its actual complete original row")?;
        let inputs = rows
            .body_inputs
            .get(reference.0 as usize)
            .filter(|input| input.body == reference)
            .ok_or("source provided body omitted its actual raw input row")?;
        if catalog.origin(body)? != origin
            || inputs.frame != function.original.binder
            || header.constant
            || header.universals != 0
            || function.parent.is_some()
            || header.signature != header.callable
            || raw.context != s::CallContext::Runtime
        {
            return Err(
                "source original requires its complete supported ordinary signature/context",
            );
        }
        if let Some(builtins) = &inputs.builtins
            && builtins.environment != s::EnvironmentOrigin::Absent
            && builtins.environment != inputs.environment
        {
            return Err(
                "source original builtin environment changed its actual construction origin",
            );
        }
        let original = Original {
            graph: self,
            catalog,
            header,
            definition,
            body,
            rows,
            reference,
        };
        original.check_locals(steps)?;
        Ok(Some(original))
    }
}
impl Original<'_> {
    pub(crate) fn raw(&self) -> &s::Original {
        &self.rows.originals[self.reference.0 as usize]
    }
    pub(crate) fn inputs(&self) -> &s::BodyInputs {
        &self.rows.body_inputs[self.reference.0 as usize]
    }
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> TypeId {
        self.header.signature
    }
    pub(crate) fn source(&self) -> Result<BodyOrigin, &'static str> {
        self.catalog.origin(self.body)
    }
    pub(crate) fn environment(&self) -> &Arc<crate::types::parameters::environment::Environment> {
        &self.header.environment
    }
    pub(crate) fn ty(&self, reference: TypeRef, steps: &mut usize) -> Result<TypeId, &'static str> {
        charge(steps)?;
        self.graph
            .ty(reference)
            .map_err(|_| "source original has an unknown actual type root")
    }
    pub(crate) fn region(
        &self,
        reference: s::RegionRef,
        steps: &mut usize,
    ) -> Result<RegionId, &'static str> {
        charge(steps)?;
        self.graph
            .region(reference)
            .map_err(|_| "source original has an unknown actual region root")
    }
    pub(crate) fn predicate(
        &self,
        raw: s::Requirement,
        steps: &mut usize,
    ) -> Result<Predicate, &'static str> {
        charge(steps)?;
        self.graph.requirement(raw)
    }
    pub(crate) fn slot_type(
        &self,
        reference: s::SlotRef,
        steps: &mut usize,
    ) -> Result<TypeId, &'static str> {
        charge(steps)?;
        let Argument::Type(ty) = self
            .graph
            .slot(reference)
            .map_err(|_| "source original lost its actual formal slot")?
        else {
            return Err("source original contract parameter is not an actual type formal");
        };
        Ok(ty)
    }
    pub(crate) fn parameter(
        &self,
        reference: s::SlotRef,
        store: &crate::types::Store,
        steps: &mut usize,
    ) -> Result<ParameterId, &'static str> {
        charge(steps)?;
        self.graph.require_store(store)?;
        let Argument::Type(ty) = self
            .graph
            .slot(reference)
            .map_err(|_| "source original lost its actual formal slot")?
        else {
            return Err("source original contract parameter is not an actual type formal");
        };
        let Some(crate::types::Kind::Parameter(id)) = store.kind(ty) else {
            return Err("source original contract parameter lost its rigid identity");
        };
        Ok(id)
    }
    pub(crate) fn declaration(
        &self,
        reference: DeclarationRef,
        steps: &mut usize,
    ) -> Result<DefId, &'static str> {
        charge(steps)?;
        let descriptor = self.graph.descriptor(self.catalog)?;
        let rows = descriptor.rows();
        let raw = rows
            .declarations
            .get(reference.0 as usize)
            .ok_or("source original declaration is outside its actual provider")?;
        let definition = self
            .catalog
            .definition_at(descriptor.identity(), reference)?;
        let id = self.catalog.declaration(definition)?;
        self.catalog.validate_declaration(
            id,
            definition,
            slot_bindings::kind(raw.definition.kind),
        )?;
        Ok(definition)
    }
    pub(crate) fn site(&self, index: u32, steps: &mut usize) -> Result<BodySite, &'static str> {
        charge(steps)?;
        self.catalog.site(self.body, index)
    }
    pub(crate) fn input_site(
        &self,
        site: s::InputSite,
        steps: &mut usize,
    ) -> Result<ProofSite, &'static str> {
        charge(steps)?;
        match site {
            s::InputSite::Body(site) => self
                .catalog
                .proof_site(self.body, self.catalog.site(self.body, site)?),
            s::InputSite::Declaration { owner, site } => {
                let definition = self.declaration(owner, steps)?;
                let declaration = self.catalog.declaration(definition)?;
                let site = self.catalog.declaration_site(declaration, site)?;
                self.catalog.declaration_proof_site(declaration, site)
            }
        }
    }
    /// An unpaid goal for the actual authenticated Sized declaration, not a flag.
    pub(crate) fn sizing_requirement(
        &self,
        facts: &Declarations,
        steps: &mut usize,
    ) -> Result<crate::types::parameters::predicates::Bound, &'static str> {
        use crate::{support::bindings::LangItem, types::parameters::predicates::Bound};
        charge(steps)?;
        self.require_facts(facts)?;
        let role = facts
            .source_bindings
            .as_deref()
            .ok_or("source original sizing lacks its actual role catalog")?
            .validated_role(LangItem::Sized)?
            .ok_or("source original sizing lacks its actual Sized declaration")?;
        self.graph.role_origin(role)?;
        facts.associated.require_imported_role(role)?;
        let predicate = facts
            .generics
            .trait_self_bounds
            .get(&role.definition)
            .ok_or("source original sizing omitted its complete actual trait schema")?;
        let Bound::Trait(id) = predicate.bound else {
            return Err("source original sizing changed its actual ordinary trait goal");
        };
        let actual = facts
            .types
            .trait_application(id)
            .ok_or("source original sizing lost its actual full application")?;
        if actual.item.0 != role.definition.0
            || facts
                .types
                .arguments(actual.arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err("source original sizing belongs to a different trait/application");
        }
        Ok(predicate.bound)
    }
    pub(crate) fn copy_identity(
        &self,
        facts: &Declarations,
        claimed: Option<s::DeclarationAddress>,
        steps: &mut usize,
    ) -> Result<Option<DefId>, &'static str> {
        charge(steps)?;
        self.require_facts(facts)?;
        let role = facts
            .source_bindings
            .as_deref()
            .ok_or("source original Copy query lacks its actual role catalog")?
            .validated_role(crate::support::bindings::LangItem::Copy)?;
        if let Some(role) = role {
            self.graph.role_origin(role)?;
            facts.associated.require_imported_role(role)?;
        }
        let actual = role.map(|role| role.definition);
        let claimed = claimed
            .map(|address| {
                if address.artifact != s::ArtifactRef(0) {
                    return Err("source original Copy query belongs to a foreign artifact");
                }
                self.declaration(address.declaration, steps)
            })
            .transpose()?;
        if claimed != actual {
            return Err("source original Copy query changed its actual language declaration");
        }
        Ok(actual)
    }
    fn require_facts(&self, facts: &Declarations) -> Result<(), &'static str> {
        self.graph.require_store(&facts.types)?;
        if facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.origin_catalog())
            .is_none_or(|actual| !std::ptr::eq(actual, self.catalog))
        {
            return Err("source original query belongs to another actual semantic catalog");
        }
        Ok(())
    }
    fn check_locals(&self, steps: &mut usize) -> Result<(), &'static str> {
        let descriptor = self.graph.descriptor(self.catalog)?;
        let rows = descriptor.rows();
        for (index, local) in self.raw().locals.iter().enumerate() {
            charge(steps)?;
            let Some(reference) = local.definition else {
                continue;
            };
            let raw = rows
                .declarations
                .get(reference.0 as usize)
                .ok_or("source local definition is outside its actual declaration table")?;
            let actual = raw
                .local
                .ok_or("source local definition omitted its real body association")?;
            if actual.body != self.raw().body
                || actual.ordinal as usize != index
                || actual.scope != local.scope
                || raw.definition.parent != Some(self.raw().declaration)
                || !matches!(
                    raw.definition.kind,
                    s::DefinitionKind::ValueParameter | s::DefinitionKind::Local
                )
            {
                return Err("source local definition changed its actual original/ordinal/scope");
            }
            let definition = self.declaration(reference, steps)?;
            let id = self.catalog.declaration(definition)?;
            let kind = if matches!(
                local.kind,
                crate::support::crates::generics::LocalKind::Parameter(_)
            ) {
                DefKind::Parameter
            } else {
                DefKind::Local
            };
            self.catalog.validate_declaration(id, definition, kind)?;
        }
        Ok(())
    }
}
fn charge(steps: &mut usize) -> Result<(), &'static str> {
    *steps = steps.saturating_add(1);
    if *steps > crate::support::crates::MAX_COMPONENTS {
        return Err("source original query work budget exceeded");
    }
    Ok(())
}
