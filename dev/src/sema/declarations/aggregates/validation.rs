//! Declaration-wide proofs consume completed templates before any concrete layout expands them.
use super::{Checker, Error, NodeKey, Type};
use crate::types::aggregate::finiteness::{self, Storage};

impl Checker<'_> {
    pub(super) fn validate_storage(&mut self) -> Result<(), Error> {
        let schemas = self
            .facts
            .aggregates
            .iter()
            .map(|definition| {
                let ty = self.facts.definitions[definition.item.0 as usize]
                    .ok_or("missing storage template identity")?;
                let Some(Type::Adt(item, arguments)) = self.facts.types.kind(ty) else {
                    return Err("invalid storage template identity");
                };
                Ok((item, arguments))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        // The established concrete layout pass handles ordinary declarations.
        // Symbolic templates need a finite proof before instance expansion.
        if schemas
            .iter()
            .all(|(_, args)| self.facts.types.region_args(*args).is_some())
        {
            return Ok(());
        }
        self.facts.generics.storage = Some(
            Storage::infer(&self.facts.types, &self.facts.aggregates, &schemas).map_err(
                |error| match error {
                    finiteness::Error::Invalid(message) => {
                        Error::at(crate::resolve::unit::FileId(0), 0, message)
                    }
                    finiteness::Error::Recursive(item) => {
                        let source = self.report.resolution.definitions[item.0 as usize].node;
                        source.map_or_else(
                            || {
                                Error::at(
                                    crate::resolve::unit::FileId(0),
                                    0,
                                    "recursive generic aggregate storage",
                                )
                            },
                            |key: NodeKey| self.error(key, "recursive generic aggregate storage"),
                        )
                    }
                },
            )?,
        );
        Ok(())
    }
}

impl Checker<'_> {
    pub(super) fn validate_fields(&mut self) -> Result<(), Error> {
        for index in 0..self.nominals.entries.len() {
            let item = self.nominals.entries[index].item;
            let definition = self.facts.aggregates.get(item).ok_or_else(|| {
                self.error(
                    self.nominals.entries[index].key,
                    "missing field declaration",
                )
            })?;
            let kind = definition.kind;
            let mut union_fields = super::unions::Fields::default();
            let packed = definition.repr.packed.is_some();
            let owner = crate::resolve::model::DefId(item.0);
            let environment = self.facts.generics.environments.get(&owner).cloned();
            let mut proof = self.declaration_proof(owner, environment);
            self.validate_bounds(owner, &mut proof)?;
            for ordinal in 0..self.facts.members.variants(item).len() {
                let id = self.facts.members.variants(item)[ordinal];
                let variant = self.facts.members.variants[id.0 as usize];
                for position in variant.start..variant.end {
                    let field = self.facts.members.fields[position as usize];
                    let source = field
                        .origin
                        .require_diagnostic()
                        .map_err(|e| self.error(self.nominals.entries[index].key, e))?;
                    let properties = proof
                        .check(&mut self.facts, field.ty)
                        .map_err(|e| self.error(source, e))?;
                    if !properties.well_formed
                        || !properties.sized
                            && (kind != crate::types::aggregate::Kind::Struct
                                || position + 1 != variant.end)
                    {
                        return Err(
                            self.error(source, "only a struct's final field may be unsized")
                        );
                    }
                    if packed && !properties.sized {
                        self.validate_packed_tail(owner, field)?;
                    }
                    if kind == crate::types::aggregate::Kind::Union {
                        self.validate_union_field(field, &mut proof, &mut union_fields)?;
                    }
                }
            }
            self.retain_trait_proof(owner, self.nominals.entries[index].key, &mut proof);
        }
        Ok(())
    }

    fn validate_packed_tail(
        &mut self,
        owner: crate::resolve::model::DefId,
        field: crate::sema::members::Field,
    ) -> Result<(), Error> {
        let source = field
            .origin
            .require_diagnostic()
            .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        let mut drops =
            crate::types::destruction::NeedsDrop::new(self.facts.destructors.declared());
        if let Some(environment) = self.facts.generics.environments.get(&owner) {
            drops = drops.scoped(std::sync::Arc::clone(environment));
        }
        self.facts
            .aggregates
            .prepare(&mut self.facts.types, field.ty)
            .map_err(|e| self.error(source, e))?;
        if drops
            .demand(&self.facts.types, &self.facts.aggregates, field.ty)
            .map_err(|e| self.error(source, e))?
        {
            return Err(self.error(source, "an unsized packed tail must not need destruction"));
        }
        Ok(())
    }
}
