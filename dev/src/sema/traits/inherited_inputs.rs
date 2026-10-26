//! Source ownership stays unchanged; imported inheritance is an indexed prefix.
use super::Located;
use crate::{
    resolve::{
        body_origin::{Catalog, ImportedDeclarationId},
        model::{DefId, DefKind},
        unit::NodeKey,
    },
    sema::declaration_inputs::{Input, Inputs},
    support::hash::HashSet,
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../tests/invariants/prefix_inputs.rs"]
mod tests;

pub(in crate::sema) struct Source {
    pub self_bounds: HashSet<NodeKey>,
    pub predicates: Vec<Located>,
}
pub(in crate::sema) struct Imported {
    declaration: ImportedDeclarationId,
    /// Shared with the completed declaration header; only the prefix is used
    /// for trait requirements, while its suffix remains in the Environment.
    inputs: Arc<[Input]>,
    self_subjects: Box<[bool]>,
}
pub(in crate::sema) enum HeaderInputs {
    Source(Source),
    Imported(Imported),
}
impl HeaderInputs {
    pub(in crate::sema) fn source(self_bounds: HashSet<NodeKey>, predicates: Vec<Located>) -> Self {
        Self::Source(Source {
            self_bounds,
            predicates,
        })
    }
    pub(in crate::sema) fn source_ref(&self) -> Result<&Source, &'static str> {
        let Self::Source(source) = self else {
            return Err("imported trait inputs have no source syntax inventory");
        };
        Ok(source)
    }
    pub(in crate::sema) fn source_mut(&mut self) -> Result<&mut Source, &'static str> {
        let Self::Source(source) = self else {
            return Err("imported trait inputs cannot acquire source predicates");
        };
        Ok(source)
    }
    pub(in crate::sema) fn view(&self) -> Inputs<'_> {
        match self {
            Self::Source(source) => Inputs::new(&source.predicates, &[]),
            Self::Imported(imported) => {
                Inputs::new(&[], &imported.inputs[..imported.self_subjects.len()])
            }
        }
    }
    pub(in crate::sema) fn is_self(
        &self,
        index: usize,
        input: Input,
    ) -> Result<bool, &'static str> {
        match self {
            Self::Source(source) => Ok(input
                .source
                .source()
                .is_some_and(|site| source.self_bounds.contains(&site))),
            Self::Imported(imported) => {
                if imported.inputs.get(index).is_none_or(|actual| {
                    actual.source != input.source || actual.value != input.value
                }) {
                    return Err("imported Self classification changed its exact canonical input");
                }
                imported
                    .self_subjects
                    .get(index)
                    .copied()
                    .ok_or("imported Self classification is outside the canonical prefix")
            }
        }
    }
    /// Consume the exact typed remapper's completed inputs. No caller can pass
    /// arbitrary predicates carrying an otherwise valid declaration site.
    pub(in crate::sema) fn imported(prepared: crate::sema::crates::PreparedSourceInputs) -> Self {
        let (declaration, inputs, self_subjects) = prepared.into_parts();
        Self::Imported(Imported {
            declaration,
            inputs,
            self_subjects,
        })
    }
    /// Borrow only this exact installed imported declaration's canonical owner.
    /// This exposes data correspondence, never body or selected-instance proof.
    pub(in crate::sema) fn imported_parts(
        &self,
        catalog: &Catalog,
        definition: DefId,
    ) -> Result<(&Arc<[Input]>, &[bool]), &'static str> {
        self.require_catalog(catalog, definition)?;
        let Self::Imported(imported) = self else {
            return Err("source trait inputs cannot supply an imported canonical owner");
        };
        Ok((&imported.inputs, &imported.self_subjects))
    }
    pub(in crate::sema) fn require_catalog(
        &self,
        catalog: &Catalog,
        definition: DefId,
    ) -> Result<(), &'static str> {
        let Self::Imported(imported) = self else {
            return Err("source trait inputs cannot authenticate an imported header");
        };
        catalog.validate_declaration(imported.declaration, definition, DefKind::Trait)?;
        Ok(())
    }
}
