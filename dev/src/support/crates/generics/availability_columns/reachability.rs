//! External bindings distinguish API trait impls from internal impls.
use super::super::{Columns, DefinitionKind, ModuleNamespace, ModuleVisibility, Type};
use crate::support::crates::{
    MAX_BYTES,
    generics::{TypeRef, Work, source::DeclarationAddress},
    owner_catalog::DeclarationRef,
};

pub(super) struct Reachability {
    exported: Vec<bool>,
}
impl Reachability {
    pub(super) fn scratch_bytes(columns: &Columns, namespace: &ModuleNamespace) -> usize {
        columns
            .declarations
            .len()
            .saturating_add(
                namespace
                    .modules
                    .len()
                    .saturating_mul(size_of::<DeclarationRef>()),
            )
            .saturating_add(size_of::<Self>() + size_of::<Vec<DeclarationRef>>())
    }

    pub(super) fn new(
        columns: &Columns,
        namespace: &ModuleNamespace,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        if base.saturating_add(Self::scratch_bytes(columns, namespace)) > MAX_BYTES {
            return Err("availability reachability exceeds combined storage budget");
        }
        work.charge(columns.declarations.len())?;
        let mut exported = vec![false; columns.declarations.len()];
        let mut pending = Vec::with_capacity(namespace.modules.len());
        exported[namespace.root.0 as usize] = true;
        pending.push(namespace.root);
        let bindings = &namespace.bindings;
        let search = bindings.len().checked_ilog2().unwrap_or(0) as usize + 1;
        while let Some(module) = pending.pop() {
            work.charge(search)?;
            let start = bindings.partition_point(|binding| binding.module.0 < module.0);
            for binding in bindings[start..]
                .iter()
                .take_while(|binding| binding.module == module)
            {
                work.charge(1)?;
                if binding.visibility != ModuleVisibility::Public
                    || binding.target.artifact != columns.policy.provider
                {
                    continue;
                }
                let target = binding.target.declaration;
                let index = target.0 as usize;
                if !std::mem::replace(&mut exported[index], true)
                    && columns.declarations[index].definition.kind == DefinitionKind::Module
                {
                    pending.push(target);
                }
            }
        }
        Ok(Self { exported })
    }

    pub(super) fn declaration(&self, columns: &Columns, address: DeclarationAddress) -> bool {
        address.artifact != columns.policy.provider || self.exported[address.declaration.0 as usize]
    }

    pub(super) fn subject(
        &self,
        columns: &Columns,
        mut ty: TypeRef,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        loop {
            work.charge(1)?;
            match columns.types[ty.0 as usize] {
                Type::Nominal { declaration, .. } => {
                    return Ok(self.declaration(columns, declaration));
                }
                Type::Foreign(declaration) => return Ok(self.exported[declaration.0 as usize]),
                Type::Reference { referent, .. } => ty = referent,
                Type::RawPointer { pointee, .. } => ty = pointee,
                Type::Array { element, .. } | Type::Slice(element) => ty = element,
                Type::Pattern { base, .. } => ty = base,
                _ => return Ok(true),
            }
        }
    }
}
