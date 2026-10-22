//! Artifact-local original ownership is structural data, never execution permission.
use super::{CrateIdentity, MAX_BYTES, MAX_FUNCTIONS};
use std::sync::Arc;

/// Ordinal in this artifact's complete declaration table, including private owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeclarationRef(pub u32);

/// Ordinal in one defining artifact, not a frontend body or syntax identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BodyRef(pub u32);

/// Structural body category of a declaration; signatures and permissions are separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclarationBody {
    Function,
    Constant,
    Static,
    /// A type, member or other declaration can own anonymous type-level expressions.
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyKind {
    Function,
    Constant,
    Static,
    Closure,
    Anonymous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyOwner {
    pub declaration: DeclarationRef,
    pub parent: Option<BodyRef>,
    pub kind: BodyKind,
    /// Position among this parent's direct children, or this declaration's roots.
    pub ordinal: u32,
}

/// A reference crossing an artifact boundary always retains its defining crate.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BodyIdentity {
    owner: CrateIdentity,
    body: BodyRef,
}
impl BodyIdentity {
    #[must_use]
    pub const fn owner(&self) -> &CrateIdentity {
        &self.owner
    }
    #[must_use]
    pub const fn body(&self) -> BodyRef {
        self.body
    }
}

/// Canonical immutable original-owner rows, shared by all bodies in an artifact.
/// Runtime-template availability and proof-context membership are not wire flags.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyCatalog {
    owner: CrateIdentity,
    rows: Arc<[BodyOwner]>,
}
impl BodyCatalog {
    pub const MAX_BODIES: usize = MAX_FUNCTIONS;
    pub const MAX_DECLARATIONS: usize = MAX_FUNCTIONS;

    /// Validate before publication, without allocating frontend identities.
    ///
    /// Roots precede children and are ordered by declaration and root ordinal.
    /// Children are ordered by parent body ordinal and child ordinal, yielding
    /// a canonical breadth-first forest. Each parent must precede its children.
    /// The declaration's named body, when present, is root zero. Anonymous roots
    /// also cover expressions in signatures and declarations without a named body.
    ///
    /// Declaration categories must come from the same artifact's declaration
    /// table. The importer must subsequently validate their actual remapped
    /// declarations, body contents and contracts through ordinary semantic proof.
    /// # Errors
    /// Rejects quotas, unknown/foreign parents, cycles, duplicate or reordered
    /// ordinals and body kinds incompatible with their structural owner.
    pub fn new(
        owner: CrateIdentity,
        declarations: &[DeclarationBody],
        rows: Vec<BodyOwner>,
    ) -> Result<Self, &'static str> {
        preflight(declarations.len(), rows.len(), rows.capacity())?;
        validate(declarations, &rows, || {})?;
        Ok(Self {
            owner,
            rows: rows.into(),
        })
    }
    #[must_use]
    pub const fn owner(&self) -> &CrateIdentity {
        &self.owner
    }
    #[must_use]
    pub fn rows(&self) -> &[BodyOwner] {
        &self.rows
    }
    /// Local references are meaningful only with the catalog that supplied them.
    #[must_use]
    pub fn row(&self, reference: BodyRef) -> Option<&BodyOwner> {
        self.rows.get(reference.0 as usize)
    }
    #[must_use]
    pub fn identity(&self, reference: BodyRef) -> Option<BodyIdentity> {
        self.row(reference).map(|_| BodyIdentity {
            owner: self.owner.clone(),
            body: reference,
        })
    }
    /// Equal ordinals in another crate never denote this catalog's body.
    #[must_use]
    pub fn lookup(&self, identity: &BodyIdentity) -> Option<&BodyOwner> {
        (identity.owner == self.owner)
            .then(|| self.row(identity.body))
            .flatten()
    }
    #[must_use]
    pub fn shares_storage(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.rows, &other.rows)
    }
}

fn preflight(declarations: usize, rows: usize, capacity: usize) -> Result<(), &'static str> {
    if declarations > BodyCatalog::MAX_DECLARATIONS || rows > BodyCatalog::MAX_BODIES {
        return Err("metadata body ownership exceeds row budget");
    }
    // Conversion to Arc can coexist with the input vector. Validation itself
    // uses constant scratch and never allocates one map or ancestry list per body.
    let peak = capacity
        .checked_add(rows)
        .and_then(|count| count.checked_mul(size_of::<BodyOwner>()))
        .and_then(|bytes| bytes.checked_add(size_of::<BodyCatalog>() + 2 * size_of::<usize>()));
    if peak.is_none_or(|bytes| bytes > MAX_BYTES) {
        return Err("metadata body ownership exceeds storage budget");
    }
    Ok(())
}

fn validate(
    declarations: &[DeclarationBody],
    rows: &[BodyOwner],
    mut visit: impl FnMut(),
) -> Result<(), &'static str> {
    let mut previous = None;
    for (index, row) in rows.iter().enumerate() {
        visit();
        let declaration = declarations
            .get(row.declaration.0 as usize)
            .ok_or("unknown metadata body declaration")?;
        let group = if let Some(parent) = row.parent {
            visit();
            let parent_index = parent.0 as usize;
            if parent_index >= index {
                return Err("metadata body parent is forward or cyclic");
            }
            if rows[parent_index].declaration != row.declaration {
                return Err("metadata body parent belongs to another declaration");
            }
            if !matches!(row.kind, BodyKind::Closure | BodyKind::Anonymous) {
                return Err("named metadata body cannot be a nested body");
            }
            (true, parent.0)
        } else {
            root_kind(*declaration, *row)?;
            (false, row.declaration.0)
        };
        canonical_ordinal(previous, group, row.ordinal)?;
        previous = Some((group, row.ordinal));
    }
    Ok(())
}

fn root_kind(declaration: DeclarationBody, row: BodyOwner) -> Result<(), &'static str> {
    if row.kind == BodyKind::Anonymous {
        return Ok(());
    }
    let matching = matches!(
        (declaration, row.kind),
        (DeclarationBody::Function, BodyKind::Function)
            | (DeclarationBody::Constant, BodyKind::Constant)
            | (DeclarationBody::Static, BodyKind::Static)
    );
    if !matching || row.ordinal != 0 {
        return Err("metadata body kind does not match its declaration root");
    }
    Ok(())
}

fn canonical_ordinal(
    previous: Option<((bool, u32), u32)>,
    group: (bool, u32),
    ordinal: u32,
) -> Result<(), &'static str> {
    let expected = match previous {
        Some((before, last)) if before == group => last.checked_add(1),
        Some((before, _)) if before > group => {
            return Err("metadata body owners are not in canonical order");
        }
        _ => Some(0),
    };
    if expected != Some(ordinal) {
        return Err("metadata body ordinal is duplicate or noncontiguous");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/imported_body_owners.rs"]
mod tests;
