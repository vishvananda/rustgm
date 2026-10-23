//! Summaries validate shared type ownership once, not once per body/use.
use super::{BinderKind, Convention, Declaration, Range, Rows, Signature, Type, TypeRef, Work};
use crate::support::{crates::owner_catalog::DeclarationRef, hash::HashSet};

pub(super) struct Graph {
    owners: Vec<Option<DeclarationRef>>,
}
impl Range {
    /// Borrow only a complete in-bounds table interval.
    /// # Errors
    /// Rejects reversed and foreign table coordinates.
    pub fn slice<T>(self, values: &[T]) -> Result<&[T], &'static str> {
        values
            .get(self.start as usize..self.end as usize)
            .ok_or("generic metadata range is invalid")
    }
}
impl Graph {
    pub(super) fn owner(&self, ty: TypeRef) -> Result<Option<DeclarationRef>, &'static str> {
        self.owners
            .get(ty.0 as usize)
            .copied()
            .ok_or("generic type root is unknown")
    }
    pub(super) fn applications(
        &mut self,
        rows: &Rows,
        owners: &[Option<DeclarationRef>],
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let first = self.owners.len();
        if rows.types.len() != first.saturating_add(owners.len().saturating_mul(2)) {
            return Err("generic application recipe table is incomplete");
        }
        for (index, &owner) in owners.iter().enumerate() {
            work.charge(2)?;
            let reference =
                super::ApplicationRef(u32::try_from(index).map_err(|_| "too many applications")?);
            if rows.types[first + 2 * index] != Type::FunctionItem(reference)
                || rows.types[first + 2 * index + 1] != Type::ApplicationSignature(reference)
            {
                return Err("generic application type recipes are reordered or foreign");
            }
            self.owners.extend([owner, owner]);
        }
        Ok(())
    }
    pub(super) fn root(
        &self,
        owner: DeclarationRef,
        ty: TypeRef,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.roots = work.roots.saturating_add(1);
        work.charge(1)?;
        let actual = self
            .owners
            .get(ty.0 as usize)
            .ok_or("generic type root is unknown")?;
        if actual.is_some_and(|actual| actual != owner) {
            return Err("generic type root captures a foreign declaration parameter");
        }
        Ok(())
    }
    pub(super) fn signature(rows: &Rows, ty: TypeRef) -> Result<Signature, &'static str> {
        match rows.types.get(ty.0 as usize) {
            Some(Type::Function(signature)) => Ok(*signature),
            _ => Err("generic function header does not name a function type"),
        }
    }
}
#[derive(PartialEq, Eq, Hash)]
enum Key<'a> {
    Scalar(u8),
    Parameter(DeclarationRef),
    Function {
        binder: super::BinderRef,
        parameters: &'a [TypeRef],
        result: TypeRef,
        unsafe_: bool,
        convention: u8,
        variadic: bool,
    },
}
pub(super) fn validate(rows: &Rows, work: &mut Work) -> Result<Graph, &'static str> {
    let header_count = rows
        .calls
        .as_ref()
        .map_or(rows.types.len(), |c| c.header_types as usize);
    if header_count > rows.types.len() {
        return Err("generic header type prefix is outside its graph");
    }
    let mut keys = HashSet::with_capacity_and_hasher(
        rows.types.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    let mut owners: Vec<Option<DeclarationRef>> = Vec::with_capacity(rows.types.len());
    let mut depths: Vec<usize> = Vec::with_capacity(rows.types.len());
    let mut parameters = vec![false; rows.declarations.len()];
    for (index, node) in rows.types.iter().take(header_count).enumerate() {
        work.charge(1)?;
        let (owner, depth, key) = match *node {
            Type::Scalar(scalar) => (None, 0, Key::Scalar(scalar as u8)),
            Type::Parameter(declaration) => {
                let Some(Declaration::TypeParameter { owner, .. }) =
                    rows.declarations.get(declaration.0 as usize)
                else {
                    return Err("generic type parameter names another declaration kind");
                };
                parameters[declaration.0 as usize] = true;
                (Some(*owner), 0, Key::Parameter(declaration))
            }
            Type::Function(signature) => {
                if signature.convention != Convention::Rust || signature.variadic {
                    return Err(
                        "generic non-Rust callable types are outside this operation family",
                    );
                }
                let binder = rows
                    .binders
                    .get(signature.binder.0 as usize)
                    .ok_or("function type binder is unknown")?;
                if binder.kind == BinderKind::Clause {
                    return Err("function type uses a declaration clause binder");
                }
                let parameters = signature.parameters.slice(&rows.roots)?;
                let mut owner = None;
                let mut depth = 0;
                for &child in parameters.iter().chain(std::iter::once(&signature.result)) {
                    work.charge(1)?;
                    if child.0 as usize >= index {
                        return Err("generic type graph has a forward or cyclic edge");
                    }
                    let child_owner = owners[child.0 as usize];
                    if child_owner.is_some_and(|actual| actual != binder.owner) {
                        return Err("function type captures a foreign parameter owner");
                    }
                    owner = owner.or(child_owner);
                    depth = depth.max(depths[child.0 as usize] + 1);
                    if let Type::Function(nested) = rows.types[child.0 as usize]
                        && rows.binders[nested.binder.0 as usize].parent != Some(signature.binder)
                    {
                        return Err("nested callable changed its actual containing binder");
                    }
                }
                let convention = match signature.convention {
                    Convention::Rust => 0,
                    Convention::RustCall => 1,
                    Convention::C => 2,
                };
                (
                    owner,
                    depth,
                    Key::Function {
                        binder: signature.binder,
                        parameters,
                        result: signature.result,
                        unsafe_: signature.unsafe_,
                        convention,
                        variadic: signature.variadic,
                    },
                )
            }
            Type::FunctionItem(_) | Type::ApplicationSignature(_) => {
                return Err("generic application type appears in the header prefix");
            }
        };
        if depth > 64 {
            return Err("generic type nesting exceeds metadata depth budget");
        }
        if !keys.insert(key) {
            return Err("duplicate generic type graph node");
        }
        owners.push(owner);
        depths.push(depth);
    }
    for (index, declaration) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        if matches!(declaration, Declaration::TypeParameter { .. }) && !parameters[index] {
            return Err("generic type graph omitted a declared rigid parameter root");
        }
    }
    Ok(Graph { owners })
}

pub(super) fn scratch(count: usize) -> usize {
    count.saturating_mul(
        4 * size_of::<Key<'static>>() + size_of::<Option<DeclarationRef>>() + size_of::<usize>(),
    )
}
