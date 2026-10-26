//! Namespace-resolved associated declarations; this inventory is not a type or impl proof.
mod comparisons;
mod language;
mod matching;
mod provenance;
mod reserved_owners;
pub(in crate::sema) use provenance::SourceOwner;
pub(in crate::sema) use reserved_owners::ImportedOwner;
#[cfg(test)]
#[path = "../../tests/invariants/owner_slots.rs"]
mod tests;
use crate::support::hash::HashMap;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Namespace},
        package::Error,
        unit::{FileId, NodeKey},
    },
    syntax::{ast::Kind, item::ItemKind},
    types::TypeId,
};
pub(crate) use comparisons::LibraryDefault;

/// Requirements on a projected result, distinct from assumptions about its inputs.
pub struct TypeContract {
    pub projection: TypeId,
    pub bounds: Vec<crate::types::parameters::clauses::Clause>,
}

/// A member captures its parent slots and then declares its own early lifetimes.
/// These input relations do not promise any property of the projected result.
pub struct Family {
    pub captures: u32,
    pub total: u32,
    pub regions: Vec<crate::types::outlives::Pair>,
    /// None distinguishes an unfinished declaration from an empty input contract.
    pub requirements: Option<Vec<crate::types::parameters::clauses::Clause>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OwnerId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerKind {
    Trait {
        definition: DefId,
        auto_: bool,
        unsafe_: bool,
        constant: bool,
    },
    Impl {
        target: NodeKey,
        trait_type: Option<NodeKey>,
        unsafe_: bool,
        constant: bool,
        negative: bool,
    },
    ImportedImpl(crate::resolve::body_origin::ImportedDeclarationId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Owner {
    pub source: Option<NodeKey>,
    pub self_definition: DefId,
    pub kind: OwnerKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Member {
    pub definition: DefId,
    pub owner: OwnerId,
}
#[derive(Clone, Copy)]
pub struct Receiver {
    pub binding: NodeKey,
    pub definition: DefId,
    pub ty: TypeId,
    /// Terminal Self-shaped type after the receiver's reference wrappers.
    pub base: TypeId,
    /// Built-in reference layers between the receiver and its Self-shaped base.
    pub references: u32,
    /// Binding mutability, independent of the reference's referent mutability.
    pub mutable: bool,
}

/// An elaborated owner environment, separate from method conformance and selection.
pub struct Schema {
    pub self_type: TypeId,
    pub captures: u32,
    pub bounds: Vec<crate::types::outlives::Pair>,
}

#[derive(Default)]
pub struct Inventory {
    language: Option<Box<language::Declarations>>,
    owners: Vec<Owner>,
    members: Vec<Member>,
    owner_members: Vec<Vec<DefId>>,
    declarations: HashMap<DefId, OwnerId>,
    schemas: HashMap<OwnerId, Schema>,
    pub(in crate::sema) types: HashMap<DefId, TypeContract>,
    pub(in crate::sema) families: HashMap<DefId, Family>,
    type_members: HashMap<(OwnerId, DefId), DefId>,
}
impl Inventory {
    pub(crate) fn has_associated_types(&self) -> bool {
        !self.types.is_empty()
    }
    /// Resolved namespace identity only; selection and conformance remain separate proofs.
    pub(in crate::sema) fn type_definition(
        &self,
        owner: DefId,
        declaration: DefId,
    ) -> Option<DefId> {
        self.type_members
            .get(&(self.owner(owner)?, declaration))
            .copied()
    }
    #[must_use]
    pub fn type_contract(&self, declaration: DefId) -> Option<&TypeContract> {
        self.types.get(&declaration)
    }
    /// Completed declaration inputs are data, not proof of an instantiated use.
    pub(crate) fn require_projection_header(
        &self,
        declaration: DefId,
    ) -> Result<&TypeContract, &'static str> {
        let id = self
            .owner(declaration)
            .ok_or("projected cleanup member owner is pending")?;
        let owner = self
            .get(id)
            .ok_or("projected cleanup member has an invalid owner")?;
        let OwnerKind::Trait { definition, .. } = owner.kind else {
            return Err("projected cleanup requires its actual trait member");
        };
        if self.owner(definition) != Some(id) || !self.schemas.contains_key(&id) {
            return Err("projected cleanup trait schema is pending or mismatched");
        }
        let contract = self
            .type_contract(declaration)
            .ok_or("projected cleanup type contract is pending")?;
        if self
            .families
            .get(&declaration)
            .is_some_and(|family| family.requirements.is_none())
        {
            return Err("projected cleanup family input requirements are pending");
        }
        Ok(contract)
    }
    #[must_use]
    pub fn owners(&self) -> &[Owner] {
        &self.owners
    }
    /// Declaration order is independent of the lookup table's iteration order.
    #[must_use]
    pub fn members(&self) -> &[Member] {
        &self.members
    }
    /// Direct members in resolver order, without scanning other trait or impl owners.
    #[must_use]
    pub fn owner_members(&self, owner: OwnerId) -> Option<&[DefId]> {
        self.owner_members.get(owner.0 as usize).map(Vec::as_slice)
    }
    #[must_use]
    pub fn owner(&self, definition: DefId) -> Option<OwnerId> {
        self.declarations.get(&definition).copied()
    }
    #[must_use]
    pub fn get(&self, id: OwnerId) -> Option<&Owner> {
        self.owners.get(id.0 as usize)
    }
    #[must_use]
    pub fn schema(&self, definition: DefId) -> Option<&Schema> {
        if let Some(schema) = self.language.as_ref().and_then(|m| {
            m.schemas
                .get(m.inherent_members.get(&definition).unwrap_or(&definition))
        }) {
            return Some(schema);
        }
        self.schemas.get(&self.owner(definition)?)
    }
    #[must_use]
    pub fn trait_owner(&self, member: DefId) -> Option<DefId> {
        if let Some(owner) = self.language.as_ref().and_then(|m| m.members.get(&member)) {
            return Some(*owner);
        }
        match self.get(self.owner(member)?)?.kind {
            OwnerKind::Trait { definition, .. } => Some(definition),
            OwnerKind::Impl { .. } | OwnerKind::ImportedImpl(_) => None,
        }
    }
    pub(crate) fn constant_trait(&self, definition: DefId) -> Option<bool> {
        match self.get(self.owner(definition)?)?.kind {
            OwnerKind::Trait {
                definition: actual,
                constant,
                ..
            } if actual == definition => Some(constant),
            _ => None,
        }
    }
    pub(super) fn declare(
        &mut self,
        self_definition: DefId,
        schema: Schema,
    ) -> Result<(), &'static str> {
        let owner = self
            .owner(self_definition)
            .ok_or("missing associated owner")?;
        if self.owners[owner.0 as usize].self_definition != self_definition {
            return Err("associated schema requires its Self identity");
        }
        match self.schemas.entry(owner) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(schema);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                Err("associated owner environment already completed")
            }
        }
    }
}

impl super::model::Declarations {
    /// Seal anonymous impl inputs before any prepared header can be selected.
    pub(in crate::sema) fn finish_impl_regions(
        &mut self,
        definition: DefId,
        captures: u32,
    ) -> Result<(), &'static str> {
        let owner = self
            .associated
            .owner(definition)
            .ok_or("missing impl region owner")?;
        let declaration = self
            .associated
            .get(owner)
            .ok_or("invalid impl region owner")?;
        if declaration.self_definition != definition
            || !matches!(
                declaration.kind,
                OwnerKind::Impl {
                    trait_type: Some(_),
                    ..
                }
            )
        {
            return Err("impl region completion requires the trait impl Self owner");
        }
        if self
            .generics
            .implementations
            .as_ref()
            .is_some_and(|registry| registry.header(definition).is_some())
        {
            return Err("impl regions cannot change after header publication");
        }
        let schema = self
            .associated
            .schemas
            .get_mut(&owner)
            .ok_or("missing pending impl schema")?;
        if self
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            != Some(schema.self_type)
            || captures < schema.captures
            || captures > u32::from(u16::MAX) + 1
        {
            return Err("stale or inconsistent pending impl region schema");
        }
        schema.captures = captures;
        Ok(())
    }
}

/// Retain each trait/impl owner and its direct associated declarations once.
/// Nested items belong to their own lexical scopes, never their enclosing method's owner.
/// Generic parameters and Self identities are not associated members.
/// # Errors
/// Rejects unresolved input, inconsistent resolver ownership or exhausted identities.
pub fn collect(report: &Report) -> Result<Inventory, Error> {
    if !report.names_resolved {
        return Err(Error::at(
            FileId(0),
            0,
            "associated declarations require resolved names",
        ));
    }
    let mut result = Inventory::default();
    let mut scopes = HashMap::default();
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if definition.kind != DefKind::SelfType {
            continue;
        }
        // Exact canonical owners retain typed schemas instead of source AST ownership.
        if library_owner(report, index) {
            continue;
        }
        if definition.node.is_none() && provenance::imported_self(report, index)? {
            continue;
        }
        let source = definition
            .node
            .ok_or_else(|| Error::at(FileId(0), 0, "associated Self definition has no source"))?;
        let owner = OwnerId(u32::try_from(result.owners.len()).map_err(|_| {
            super::model::Declarations::error(report, source, "too many associated owners")
        })?);
        let self_definition = identity(report, source, index)?;
        let kind = owner_kind(report, source, definition.scope)?;
        if scopes.insert(definition.scope, owner).is_some() {
            return Err(super::model::Declarations::error(
                report,
                source,
                "associated scope has multiple Self owners",
            ));
        }
        if let OwnerKind::Trait { definition, .. } = kind {
            result.declarations.insert(definition, owner);
        }
        result.declarations.insert(self_definition, owner);
        result.owners.push(Owner {
            source: Some(source),
            self_definition,
            kind,
        });
        result.owner_members.push(Vec::new());
    }
    if scopes.is_empty() {
        return Ok(result);
    }
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        let Some(&owner) = scopes.get(&definition.scope) else {
            continue;
        };
        if matches!(
            definition.kind,
            DefKind::SelfType | DefKind::Lifetime | DefKind::Generic
        ) {
            continue;
        }
        let source = result.owners[owner.0 as usize]
            .source_owner()
            .map_err(Error::unlocated)?
            .source;
        let id = identity(report, source, index)?;
        if definition.kind == DefKind::TypeAlias {
            result.type_members.insert((owner, id), id);
        }
        result.declarations.insert(id, owner);
        result.members.push(Member {
            definition: id,
            owner,
        });
        result.owner_members[owner.0 as usize].push(id);
    }
    Ok(result)
}
fn library_owner(report: &Report, index: usize) -> bool {
    report
        .resolution
        .library_types
        .slice_length()
        .is_some_and(|length| length.owner.0 as usize == index)
        || [
            crate::support::intrinsics::Intrinsic::ConstPointerAdd,
            crate::support::intrinsics::Intrinsic::MutPointerAdd,
        ]
        .into_iter()
        .any(|kind| {
            report
                .resolution
                .intrinsics
                .get(kind)
                .is_some_and(|d| d.owner.is_some_and(|owner| owner.0 as usize == index))
        })
        || report
            .resolution
            .builtin_traits
            .iteration()
            .is_some_and(|iteration| {
                iteration
                    .blankets
                    .iter()
                    .any(|d| d.definition.0 as usize == index)
            })
        || report
            .resolution
            .builtin_traits
            .collections()
            .is_some_and(|collections| {
                collections
                    .declarations()
                    .any(|d| d.definition.0 as usize == index)
            })
        || report.resolution.builtin_traits.ranges().is_some_and(|r| {
            r.implementations
                .iter()
                .any(|d| d.definition.0 as usize == index)
        })
        || report
            .resolution
            .builtin_traits
            .cloning()
            .and_then(|cloning| cloning.implementation)
            .is_some_and(|implementation| implementation.definition.0 as usize == index)
}
fn identity(report: &Report, source: NodeKey, index: usize) -> Result<DefId, Error> {
    u32::try_from(index).map(DefId).map_err(|_| {
        super::model::Declarations::error(report, source, "too many associated declarations")
    })
}
fn owner_kind(
    report: &Report,
    source: NodeKey,
    scope: crate::resolve::model::ScopeId,
) -> Result<OwnerKind, Error> {
    let error = |message| super::model::Declarations::error(report, source, message);
    let ast = &report.package.unit(source.file).configured.ast;
    let Kind::Item(index) = ast.node(source.node).kind else {
        return Err(error("associated owner has no item syntax"));
    };
    match ast.items[index as usize].kind {
        ItemKind::Trait {
            auto_,
            unsafe_,
            constant,
            ..
        } => {
            let module = report.resolution.scopes[scope.0 as usize].module;
            let definition = *report
                .resolution
                .item_definitions
                .get(&(module, source.node, Namespace::Type))
                .ok_or_else(|| error("missing trait declaration identity"))?;
            if auto_
                && !report
                    .resolution
                    .source_bindings
                    .as_deref()
                    .and_then(|r| r.language(crate::support::bindings::LangItem::Freeze))
                    .is_some_and(|actual| actual.definition == definition)
            {
                return Err(error("custom source auto traits are not implemented"));
            }
            Ok(OwnerKind::Trait {
                definition,
                auto_,
                unsafe_,
                constant,
            })
        }
        ItemKind::Impl {
            unsafe_,
            constant,
            negative,
            trait_path,
            ty,
            ..
        } => Ok(OwnerKind::Impl {
            target: NodeKey { node: ty, ..source },
            trait_type: trait_path.map(|node| NodeKey { node, ..source }),
            unsafe_,
            constant,
            negative,
        }),
        ItemKind::TraitAlias { .. } => {
            Err(error("trait alias contract expansion is not implemented"))
        }
        _ => Err(error("Self definition is not owned by a trait or impl")),
    }
}
