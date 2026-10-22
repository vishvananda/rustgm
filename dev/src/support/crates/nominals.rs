//! Artifact-owned named records retain visibility and one shared type graph.
use super::{
    Constant, CrateIdentity, ExternalDef, ExternalKind, Function, Lifetime, MAX_COMPONENTS,
    MAX_NAME_BYTES, Storage, TypeGraph, TypeNode, TypeRef,
};
use crate::support::hash::HashSet;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RecordRef(pub u16);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordField {
    name: String,
    ty: TypeRef,
    public: bool,
}
impl RecordField {
    /// Retain a canonical field name and its artifact-local type root.
    /// # Errors
    /// Rejects invalid identifiers before importing names into a frontend.
    pub fn new(name: String, ty: TypeRef, public: bool) -> Result<Self, &'static str> {
        super::identity::validate_component(&name)?;
        Ok(Self { name, ty, public })
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn ty(&self) -> TypeRef {
        self.ty
    }
    #[must_use]
    pub const fn public(&self) -> bool {
        self.public
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    definition: ExternalDef,
    public: bool,
    fields: Box<[RecordField]>,
}
impl Record {
    /// Preserve declaration order and private layout fields without syntax nodes.
    /// # Errors
    /// Rejects wrong identities, duplicate fields and oversized field tables.
    pub fn new(
        definition: ExternalDef,
        public: bool,
        fields: Vec<RecordField>,
    ) -> Result<Self, &'static str> {
        if definition.kind() != ExternalKind::Record || fields.len() > MAX_COMPONENTS {
            return Err("invalid metadata named record");
        }
        let mut names = HashSet::with_capacity_and_hasher(
            fields.len(),
            crate::support::hash::BuildFxHasher::default(),
        );
        if fields.iter().any(|field| !names.insert(field.name())) {
            return Err("duplicate metadata record field");
        }
        Ok(Self {
            definition,
            public,
            fields: fields.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn definition(&self) -> &ExternalDef {
        &self.definition
    }
    #[must_use]
    pub const fn public(&self) -> bool {
        self.public
    }
    #[must_use]
    pub fn fields(&self) -> &[RecordField] {
        &self.fields
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Nominals {
    types: TypeGraph,
    records: Box<[Record]>,
}
impl Nominals {
    /// Validate record ordinals, static field roots and finite by-value storage.
    /// Record ordinals refer to this already sorted table; it is never reordered.
    /// # Errors
    /// Rejects noncanonical identities, invalid roots, cycles and profile bounds.
    pub fn new(types: TypeGraph, records: Vec<Record>) -> Result<Self, &'static str> {
        if records.is_empty() {
            return Err("nominal metadata requires a bounded nonempty record table");
        }
        Self::for_storage(types, records)
    }
    /// Storage-only artifacts own a real type graph without inventing a nominal.
    /// # Errors
    /// Rejects malformed graphs, fields and cyclic by-value record storage.
    pub fn for_storage(types: TypeGraph, records: Vec<Record>) -> Result<Self, &'static str> {
        if records.len() > TypeGraph::MAX_NODES {
            return Err("metadata record table exceeds profile budget");
        }
        if records
            .windows(2)
            .any(|pair| pair[0].definition >= pair[1].definition)
        {
            return Err("metadata record identities are duplicate or noncanonical");
        }
        let mut field_types = Vec::with_capacity(types.nodes().len());
        for node in types.nodes() {
            let valid = match *node {
                TypeNode::Scalar(_) => true,
                TypeNode::Record(record) => {
                    if usize::from(record.0) >= records.len() {
                        return Err("unknown metadata record ordinal");
                    }
                    true
                }
                TypeNode::Reference {
                    region,
                    mutable,
                    referent,
                } => region == Lifetime::Static && !mutable && field_types[usize::from(referent.0)],
            };
            field_types.push(valid);
        }
        let mut fields = 0usize;
        for record in &records {
            fields = fields.saturating_add(record.fields.len());
            if fields > MAX_COMPONENTS {
                return Err("metadata field table exceeds profile budget");
            }
            for field in record.fields() {
                if field_types.get(usize::from(field.ty.0)) != Some(&true) {
                    return Err("metadata record field is outside the static shared profile");
                }
            }
        }
        validate_storage(&types, &records)?;
        Ok(Self {
            types,
            records: records.into_boxed_slice(),
        })
    }
    #[must_use]
    pub const fn types(&self) -> &TypeGraph {
        &self.types
    }
    #[must_use]
    pub fn records(&self) -> &[Record] {
        &self.records
    }
    pub(super) fn validate(
        &self,
        owner: &CrateIdentity,
        functions: &[Function],
        constants: &[Constant],
        storage: Option<&Storage>,
        (mut atoms, mut names): (usize, usize),
    ) -> Result<(usize, usize), &'static str> {
        atoms = atoms
            .saturating_add(self.types.nodes().len())
            .saturating_add(self.records.len());
        for record in self.records() {
            if record.definition.owner() != owner {
                return Err("foreign definition in metadata export table");
            }
            atoms = atoms
                .saturating_add(record.definition.path().len())
                .saturating_add(record.fields.len().saturating_mul(2));
            names = names
                .saturating_add(
                    record
                        .definition
                        .path()
                        .iter()
                        .map(String::len)
                        .sum::<usize>(),
                )
                .saturating_add(
                    record
                        .fields
                        .iter()
                        .map(|field| field.name.len())
                        .sum::<usize>(),
                );
            if atoms > MAX_COMPONENTS || names > MAX_NAME_BYTES {
                return Err("metadata contents exceed profile budget");
            }
        }
        if functions
            .iter()
            .any(|function| !self.types.shares_storage(function.signature().types()))
        {
            return Err("nominal signatures must share the artifact type graph");
        }
        let roots = functions.iter().flat_map(|function| {
            function
                .signature()
                .parameters()
                .iter()
                .copied()
                .chain(std::iter::once(function.signature().result()))
        });
        self.types.canonical(
            roots
                .chain(
                    self.records
                        .iter()
                        .flat_map(|record| record.fields.iter().map(RecordField::ty)),
                )
                .chain(
                    storage
                        .into_iter()
                        .flat_map(|s| s.statics().iter().map(super::Static::ty)),
                ),
        )?;
        self.validate_reachable(functions, storage)?;
        self.validate_paths(functions, constants, storage)?;
        Ok((atoms, names))
    }
    fn validate_paths(
        &self,
        functions: &[Function],
        constants: &[Constant],
        storage: Option<&Storage>,
    ) -> Result<(), &'static str> {
        let records: HashSet<_> = self
            .records
            .iter()
            .map(|record| record.definition.path())
            .collect();
        let paths = records
            .iter()
            .copied()
            .chain(
                functions
                    .iter()
                    .map(|function| function.definition().path()),
            )
            .chain(
                constants
                    .iter()
                    .map(|constant| constant.definition().path()),
            )
            .chain(
                storage
                    .into_iter()
                    .flat_map(|s| s.statics().iter().map(|entry| entry.definition().path())),
            );
        for path in paths {
            for length in 1..path.len() {
                if records.contains(&path[..length]) {
                    return Err("metadata record conflicts with an imported module path");
                }
            }
        }
        Ok(())
    }
    fn validate_reachable(
        &self,
        functions: &[Function],
        storage: Option<&Storage>,
    ) -> Result<(), &'static str> {
        // Resolve each type's nominal leaf once; repeated roots never rescan chains.
        let mut leaves = Vec::with_capacity(self.types.nodes().len());
        for node in self.types.nodes() {
            leaves.push(match *node {
                TypeNode::Scalar(_) => None,
                TypeNode::Record(record) => Some(usize::from(record.0)),
                TypeNode::Reference { referent, .. } => leaves[usize::from(referent.0)],
            });
        }
        // This profile has no private-module or private-type API visibility
        // transport. Private declarations may occur only behind private fields.
        let public_root = |root: TypeRef| {
            leaves[usize::from(root.0)].is_none_or(|record| self.records[record].public)
        };
        if functions.iter().any(|function| {
            !public_root(function.signature().result())
                || function
                    .signature()
                    .parameters()
                    .iter()
                    .any(|&root| !public_root(root))
        }) || storage.is_some_and(|s| {
            s.statics()
                .iter()
                .filter(|entry| entry.public())
                .any(|entry| !public_root(entry.ty()))
        }) || self
            .records
            .iter()
            .filter(|record| record.public)
            .any(|record| {
                record
                    .fields
                    .iter()
                    .any(|field| field.public && !public_root(field.ty))
            })
        {
            return Err("unexported nominal in a public metadata contract is outside the profile");
        }
        let mut seen = vec![false; self.records.len()];
        let mut work: Vec<_> = self
            .records
            .iter()
            .enumerate()
            .filter_map(|(index, record)| record.public.then_some(index))
            .collect();
        work.extend(
            functions
                .iter()
                .flat_map(|function| {
                    function
                        .signature()
                        .parameters()
                        .iter()
                        .copied()
                        .chain(std::iter::once(function.signature().result()))
                })
                .filter_map(|root| leaves[usize::from(root.0)]),
        );
        work.extend(
            storage
                .into_iter()
                .flat_map(|s| s.statics().iter())
                .filter_map(|entry| leaves.get(usize::from(entry.ty().0)).copied().flatten()),
        );
        while let Some(index) = work.pop() {
            if std::mem::replace(&mut seen[index], true) {
                continue;
            }
            work.extend(
                self.records[index]
                    .fields
                    .iter()
                    .filter_map(|field| leaves[usize::from(field.ty.0)]),
            );
        }
        if seen.contains(&false) {
            return Err("metadata contains unreachable private records");
        }
        Ok(())
    }
}

fn validate_storage(types: &TypeGraph, records: &[Record]) -> Result<(), &'static str> {
    let mut incoming = vec![0usize; records.len()];
    let mut users = vec![Vec::new(); records.len()];
    for (index, record) in records.iter().enumerate() {
        for field in record.fields() {
            if let Some(TypeNode::Record(target)) = types.node(field.ty) {
                incoming[index] += 1;
                users[usize::from(target.0)].push(index);
            }
        }
    }
    let mut ready: VecDeque<_> = incoming
        .iter()
        .enumerate()
        .filter_map(|(index, &count)| (count == 0).then_some(index))
        .collect();
    let mut completed = 0;
    while let Some(index) = ready.pop_front() {
        completed += 1;
        for &user in &users[index] {
            incoming[user] -= 1;
            if incoming[user] == 0 {
                ready.push_back(user);
            }
        }
    }
    if completed != records.len() {
        return Err("cyclic by-value metadata record storage");
    }
    Ok(())
}
