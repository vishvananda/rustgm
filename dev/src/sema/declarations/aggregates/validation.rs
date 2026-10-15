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
            let owner = crate::resolve::model::DefId(item.0);
            let environment = self.facts.generics.environments.get(&owner).cloned();
            let mut proof = crate::sema::nominals::Proof::new(environment);
            self.validate_bounds(owner, &mut proof)?;
            for ordinal in 0..self.facts.members.variants(item).len() {
                let id = self.facts.members.variants(item)[ordinal];
                let variant = self.facts.members.variants[id.0 as usize];
                for field in variant.start..variant.end {
                    let field = self.facts.members.fields[field as usize];
                    let properties = proof
                        .check(&mut self.facts, field.ty)
                        .map_err(|e| self.error(field.node, e))?;
                    if !properties.well_formed || !properties.sized {
                        return Err(self.error(
                            field.node,
                            "aggregate fields must be well-formed and sized in this profile",
                        ));
                    }
                }
            }
            self.retain_trait_equalities(owner, self.nominals.entries[index].key, &mut proof);
        }
        Ok(())
    }
}
