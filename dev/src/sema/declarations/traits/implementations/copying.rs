//! Actual source Copy impls must prove their physical fields under declared premises.
use super::{Checker, Error, NodeKey};
use crate::{
    sema::nominals::Proof,
    types::{
        Kind, TypeId,
        parameters::predicates::{Bound, Predicate},
    },
};
impl Checker<'_> {
    pub(super) fn validate_source_copy_impl(
        &mut self,
        source: NodeKey,
        predicate: Predicate,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(());
        }
        let Some(declaration) = crate::sema::compiler_items::copy_declaration(&mut self.facts)
            .map_err(|e| self.error(source, e))?
        else {
            return Ok(());
        };
        let Some((application, _)) = predicate.bound.trait_parts() else {
            return Ok(());
        };
        let Some(actual) = self.facts.types.trait_application(application) else {
            return Err(self.error(source, "source Copy impl has invalid application"));
        };
        if actual.item.0 != declaration.0 {
            return Ok(());
        }
        let fields = self.source_copy_fields(source, predicate.ty)?;
        for ty in fields {
            if !proof
                .prove(
                    &mut self.facts,
                    Predicate {
                        ty,
                        bound: Bound::Trait(application),
                    },
                )
                .map_err(|e| self.error(source, e))?
            {
                return Err(self.error(
                    source,
                    "source Copy implementation requires every field to implement actual Copy",
                ));
            }
        }
        Ok(())
    }
    fn source_copy_fields(&mut self, source: NodeKey, ty: TypeId) -> Result<Vec<TypeId>, Error> {
        let error = |message| crate::sema::model::Declarations::error(self.report, source, message);
        match self
            .facts
            .types
            .kind(ty)
            .ok_or_else(|| error("invalid source Copy target"))?
        {
            Kind::Adt(item, _) => {
                if self.facts.destructors.has_declared(item) {
                    return Err(error("a type cannot implement both source Copy and Drop"));
                }
                self.facts
                    .aggregates
                    .prepare(&mut self.facts.types, ty)
                    .map_err(error)?;
                let shape = self
                    .facts
                    .aggregates
                    .raw_shape(&self.facts.types, ty)
                    .map_err(error)?;
                let mut fields = Vec::new();
                for variant in shape.variants() {
                    let values = self
                        .facts
                        .types
                        .list(variant.fields)
                        .ok_or_else(|| error("invalid source Copy fields"))?;
                    if fields.len().saturating_add(values.len()) > 1_000_000 {
                        return Err(error("source Copy field proof budget exceeded"));
                    }
                    fields.extend_from_slice(values);
                }
                Ok(fields)
            }
            Kind::Reference { mutable: true, .. } => {
                Err(error("unique references cannot implement source Copy"))
            }
            Kind::Primitive(_)
            | Kind::Pointer { .. }
            | Kind::Reference { mutable: false, .. }
            | Kind::Function { .. } => Ok(Vec::new()),
            Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } => Ok(vec![element]),
            Kind::Tuple(list) => self
                .facts
                .types
                .list(list)
                .map(<[TypeId]>::to_vec)
                .ok_or_else(|| error("invalid source Copy tuple")),
            _ => Err(error(
                "source Copy target is outside the supported implementation profile",
            )),
        }
    }
}
