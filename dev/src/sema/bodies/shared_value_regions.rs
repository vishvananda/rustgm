//! Shared wrapper views relate actual prepared fields and the real loan lifetime.
use super::{Checker, Error, NodeId};
use crate::{
    sema::{
        declarations::{ReborrowField, ReborrowStructView},
        model::{AdjustmentKind, Coercion, Outlives, ReferenceConversion},
        reborrow_adjustments::Field,
    },
    types::{Kind, TypeId, arguments::Argument},
};

impl Checker<'_> {
    pub(super) fn shared_value_regions(
        &mut self,
        node: NodeId,
        source: TypeId,
        target: TypeId,
    ) -> Result<Option<Field>, Error> {
        let source_view: ReborrowStructView =
            crate::sema::declarations::reborrow_struct_view(self.facts, source)
                .map_err(|e| self.error(node, e))?;
        let target_view: ReborrowStructView =
            crate::sema::declarations::reborrow_struct_view(self.facts, target)
                .map_err(|e| self.error(node, e))?;
        self.value_borrow_lifetimes(node, source_view.ty, target_view.ty)?;
        let field = field_pair(&source_view.fields, &target_view.fields)
            .map_err(|e| self.error(node, e))?;
        let mut next = field.map(|f| (f.source_ty, f.target_ty));
        let mut seen = crate::support::hash::HashSet::default();
        while let Some((a, b)) = next.take() {
            let growth = crate::support::retained::Growth::map::<(TypeId, TypeId), ()>(
                seen.len(),
                seen.capacity(),
                1,
            );
            self.facts
                .charge_caller_inputs(growth.work.saturating_add(1), growth.bytes)
                .map_err(|e| self.error(node, e))?;
            seen.try_reserve(1).map_err(|_| {
                self.error(node, "shared reborrow field traversal allocation failed")
            })?;
            if !seen.insert((a, b)) {
                return Err(self.error(node, "cycle in shared reborrow field relations"));
            }
            let a = self.alias_type(node, a)?;
            let b = self.alias_type(node, b)?;
            match (self.facts.types.kind(a), self.facts.types.kind(b)) {
                (Some(Kind::Adt(x, _)), Some(Kind::Adt(y, _))) if x != y => {
                    let a = crate::sema::declarations::reborrow_struct_view(self.facts, a)
                        .map_err(|e| self.error(node, e))?;
                    let b = crate::sema::declarations::reborrow_struct_view(self.facts, b)
                        .map_err(|e| self.error(node, e))?;
                    self.value_borrow_lifetimes(node, a.ty, b.ty)?;
                    next = field_pair(&a.fields, &b.fields)
                        .map_err(|e| self.error(node, e))?
                        .map(|f| (f.source_ty, f.target_ty));
                }
                (left, right) => {
                    let reference = matches!(
                        (left, right),
                        (
                            Some(Kind::Reference { mutable: true, .. }),
                            Some(Kind::Reference { mutable: false, .. })
                        )
                    )
                    .then_some(ReferenceConversion {
                        dereferences: 1,
                        unsize: false,
                        two_phase: false,
                    });
                    self.facts.retain_coercion(Coercion {
                        body: self.state.body,
                        source: self.key(node),
                        actual: a,
                        expected: b,
                        kind: AdjustmentKind::Equality,
                        reference,
                    });
                }
            }
        }
        Ok(field)
    }
    fn value_borrow_lifetimes(
        &mut self,
        node: NodeId,
        source: TypeId,
        target: TypeId,
    ) -> Result<(), Error> {
        let region = |ty| -> Result<_, &'static str> {
            let Some(Kind::Adt(_, args)) = self.facts.types.kind(ty) else {
                return Err("value reborrow lifetime requires a nominal application");
            };
            let mut regions = self
                .facts
                .types
                .arguments(args)
                .ok_or("value reborrow arguments disappeared")?
                .filter_map(|a| {
                    if let Argument::Lifetime(r) = a {
                        Some(r)
                    } else {
                        None
                    }
                });
            let region = regions
                .next()
                .ok_or("value reborrow has no actual lifetime")?;
            if regions.next().is_some() {
                return Err("value reborrow has multiple actual lifetimes");
            }
            Ok(region)
        };
        let longer = region(source).map_err(|e| self.error(node, e))?;
        let shorter = region(target).map_err(|e| self.error(node, e))?;
        self.facts.outlives.push(Outlives {
            body: self.state.body,
            source: self.key(node),
            longer,
            shorter,
        });
        Ok(())
    }
}

fn field_pair(
    source: &[ReborrowField],
    target: &[ReborrowField],
) -> Result<Option<Field>, &'static str> {
    match (source, target) {
        ([], []) => Ok(None),
        ([a], [b]) => Ok(Some(Field {
            source: a.selection,
            target: b.selection,
            source_ty: a.ty,
            target_ty: b.ty,
        })),
        _ => Err("shared reborrow changed its checked actual data fields"),
    }
}
