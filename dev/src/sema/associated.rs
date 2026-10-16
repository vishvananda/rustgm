//! Namespace-resolved associated declarations; this inventory is not a type or impl proof.
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
use std::collections::HashMap;

/// Requirements on a projected result, distinct from assumptions about its inputs.
pub struct TypeContract {
    pub projection: TypeId,
    pub bounds: Vec<crate::types::parameters::predicates::Bound>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OwnerId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnerKind {
    Trait {
        definition: DefId,
        unsafe_: bool,
    },
    Impl {
        target: NodeKey,
        trait_type: Option<NodeKey>,
        unsafe_: bool,
        negative: bool,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Owner {
    pub source: NodeKey,
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
    owners: Vec<Owner>,
    members: Vec<Member>,
    declarations: HashMap<DefId, OwnerId>,
    schemas: HashMap<OwnerId, Schema>,
    pub(in crate::sema) types: HashMap<DefId, TypeContract>,
    type_names: HashMap<DefId, crate::resolve::names::NameId>,
    type_members: HashMap<(OwnerId, crate::resolve::names::NameId), DefId>,
}
impl Inventory {
    /// Resolved namespace identity only; selection and conformance remain separate proofs.
    pub(in crate::sema) fn type_definition(
        &self,
        owner: DefId,
        declaration: DefId,
    ) -> Option<DefId> {
        let name = *self.type_names.get(&declaration)?;
        self.type_members.get(&(self.owner(owner)?, name)).copied()
    }
    #[must_use]
    pub fn type_contract(&self, declaration: DefId) -> Option<&TypeContract> {
        self.types.get(&declaration)
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
        self.schemas.get(&self.owner(definition)?)
    }
    #[must_use]
    pub fn trait_owner(&self, member: DefId) -> Option<DefId> {
        match self.get(self.owner(member)?)?.kind {
            OwnerKind::Trait { definition, .. } => Some(definition),
            OwnerKind::Impl { .. } => None,
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
    let mut scopes = HashMap::new();
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if definition.kind != DefKind::SelfType {
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
            source,
            self_definition,
            kind,
        });
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
        let source = result.owners[owner.0 as usize].source;
        let id = identity(report, source, index)?;
        if definition.kind == DefKind::TypeAlias {
            result.type_names.insert(id, definition.name);
            result.type_members.insert((owner, definition.name), id);
        }
        result.declarations.insert(id, owner);
        result.members.push(Member {
            definition: id,
            owner,
        });
    }
    Ok(result)
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
        ItemKind::Trait { unsafe_, .. } => {
            let module = report.resolution.scopes[scope.0 as usize].module;
            let definition = *report
                .resolution
                .item_definitions
                .get(&(module, source.node, Namespace::Type))
                .ok_or_else(|| error("missing trait declaration identity"))?;
            Ok(OwnerKind::Trait {
                definition,
                unsafe_,
            })
        }
        ItemKind::Impl {
            unsafe_,
            negative,
            trait_path,
            ty,
            ..
        } => Ok(OwnerKind::Impl {
            target: NodeKey { node: ty, ..source },
            trait_type: trait_path.map(|node| NodeKey { node, ..source }),
            unsafe_,
            negative,
        }),
        _ => Err(error("Self definition is not owned by a trait or impl")),
    }
}
