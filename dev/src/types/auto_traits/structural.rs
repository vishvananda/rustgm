//! Structural rules inspect storage fields; callable inputs are not stored values.
use super::{AutoTrait, Definitions, Obligation, Query, Step, Store};
use crate::types::{Kind, ListId, TypeId};

impl Query {
    pub(super) fn expand(
        &mut self,
        types: &mut Store,
        definitions: &mut Definitions,
        obligation: Obligation,
        source: bool,
    ) -> Result<Option<Step>, &'static str> {
        let Obligation { ty, marker } = obligation;
        match types.kind(ty).ok_or("invalid auto-trait subject")? {
            Kind::Never
            | Kind::Primitive(_)
            | Kind::Function { .. }
            | Kind::FunctionItem { .. } => {}
            Kind::Foreign(_) => self.nodes[self.cursor].value = false,
            Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..) | Kind::Dynamic { .. } => {
                self.waiting = Some(obligation);
                return Ok(Some(Step::Proof(obligation)));
            }
            Kind::Pattern { base, .. } => self.dependency(Obligation { ty: base, marker })?,
            Kind::Closure { environment, .. } => {
                self.dependency(Obligation {
                    ty: environment,
                    marker,
                })?;
            }
            Kind::Tuple(fields) => self.fields(types, fields, marker)?,
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element) => {
                self.dependency(Obligation {
                    ty: element,
                    marker,
                })?;
            }
            Kind::Adt(item, _) => {
                if definitions.get(item).is_none() {
                    return Ok(Some(Step::Fields(item)));
                }
                if let Some(target) = definitions.phantom_argument(types, ty)? {
                    self.dependency(Obligation { ty: target, marker })?;
                    return Ok(None);
                }
                let before = definitions.instantiation_steps();
                definitions.instantiate(types, ty)?;
                self.charge(definitions.instantiation_steps() - before)?;
                for variant in definitions.raw_shape(types, ty)?.variants() {
                    self.fields(types, variant.fields, marker)?;
                }
            }
            Kind::Reference { referent, .. } if source => {
                self.dependency(Obligation {
                    ty: referent,
                    marker,
                })?;
            }
            Kind::Pointer { pointee, .. } if source => {
                self.dependency(Obligation {
                    ty: pointee,
                    marker,
                })?;
            }
            Kind::Reference {
                referent, mutable, ..
            } => {
                self.reference(referent, mutable, marker)?;
            }
            Kind::Pointer { pointee, .. } => match marker {
                AutoTrait::Send | AutoTrait::Sync => self.nodes[self.cursor].value = false,
                AutoTrait::Unpin => {}
                AutoTrait::Freeze | AutoTrait::UnsafeUnpin => self.dependency(Obligation {
                    ty: pointee,
                    marker,
                })?,
                AutoTrait::UnwindSafe | AutoTrait::RefUnwindSafe => {
                    self.dependency(Obligation {
                        ty: pointee,
                        marker: AutoTrait::RefUnwindSafe,
                    })?;
                }
            },
        }
        Ok(None)
    }

    fn reference(
        &mut self,
        ty: TypeId,
        mutable: bool,
        marker: AutoTrait,
    ) -> Result<(), &'static str> {
        let marker = match marker {
            AutoTrait::Send if !mutable => AutoTrait::Sync,
            AutoTrait::Unpin => return Ok(()),
            AutoTrait::UnwindSafe if mutable => {
                self.nodes[self.cursor].value = false;
                return Ok(());
            }
            AutoTrait::UnwindSafe => AutoTrait::RefUnwindSafe,
            marker => marker,
        };
        self.dependency(Obligation { ty, marker })
    }

    fn fields(
        &mut self,
        types: &Store,
        fields: ListId,
        marker: AutoTrait,
    ) -> Result<(), &'static str> {
        for &ty in types.list(fields).ok_or("invalid auto-trait fields")? {
            self.dependency(Obligation { ty, marker })?;
        }
        Ok(())
    }
}
