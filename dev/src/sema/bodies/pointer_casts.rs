//! Explicit casts may reinterpret raw pointees but cannot invent pointer metadata.
use super::{Checker, Constructor, Error, Kind, NodeId, TermId, Type, TypeId, Work};
use crate::{
    abi::metadata::Metadata,
    sema::model::{AdjustmentKind, Cast, Operation},
    support::retained::Growth,
    types::infer::{Domain, Error as InferError, Solved},
};

/// A cold owned queue keeps the ordinary Work payload one pointer wide.
pub(super) type QueuedCasts = Box<Vec<NodeId>>;

impl Checker<'_> {
    /// A closure's first pass can precede the call that fixes these inputs.
    /// Its existing bottom continuation owns the cold queue, so parking the
    /// child preserves it without widening ordinary body or closure records.
    pub(super) fn defer_closure_cast(
        &mut self,
        node: NodeId,
        source: TermId,
        target: TermId,
    ) -> Result<bool, Error> {
        if !matches!(self.state.work.first(), Some(Work::CollectClosure(_))) {
            return Ok(false);
        }
        let mut pending = false;
        for term in [source, target] {
            match self.state.infer.head(&self.facts.types, term) {
                Ok(None) | Err(InferError::Normalization) => pending = true,
                Ok(Some(_)) => {}
                Err(error) => return Err(self.error(node, error)),
            }
        }
        if !pending {
            return Ok(false);
        }
        let Some(Work::CollectClosure(casts)) = self.state.work.first() else {
            return Err(self.error(node, "cast lost its closure collection frontier"));
        };
        let (length, capacity) = casts
            .as_deref()
            .map_or((0, 0), |casts| (casts.len(), casts.capacity()));
        let mut growth = Growth::vector::<NodeId>(length, capacity, 1);
        growth.bytes = growth
            .bytes
            .saturating_add(usize::from(casts.is_none()) * size_of::<Vec<NodeId>>());
        self.pay_closure_casts(node, growth, 1)?;
        let Some(Work::CollectClosure(casts)) = self.state.work.first_mut() else {
            return Err(self.error(node, "cast lost its prepaid closure frontier"));
        };
        let casts = casts.get_or_insert_with(|| Box::new(Vec::new()));
        let needed = if length == capacity {
            length.saturating_mul(2).max(4) - length
        } else {
            0
        };
        casts.try_reserve_exact(needed).map_err(|_| {
            crate::sema::model::Declarations::error(
                self.report,
                self.state.body,
                "closure cast allocation failed",
            )
        })?;
        if casts
            .capacity()
            .saturating_sub(capacity)
            .saturating_mul(size_of::<NodeId>())
            > growth.bytes
        {
            return Err(self.error(node, "closure cast allocation exceeds its prepaid bound"));
        }
        casts.push(node);
        Ok(true)
    }

    pub(super) fn resume_closure_casts(&mut self, casts: Option<QueuedCasts>) -> Result<(), Error> {
        let Some(casts) = casts else {
            return Ok(());
        };
        let node = self.state.body.node;
        let growth = Growth::vector::<Work>(
            self.state.work.len(),
            self.state.work.capacity(),
            casts.len(),
        );
        self.pay_closure_casts(node, growth, casts.len())?;
        self.state.work.try_reserve(casts.len()).map_err(|_| {
            crate::sema::model::Declarations::error(
                self.report,
                self.state.body,
                "closure cast work allocation failed",
            )
        })?;
        // Reverse insertion retains original expression order on the work stack.
        self.state.work.extend(
            (*casts)
                .into_iter()
                .rev()
                .map(|node| Work::Exit(node, None)),
        );
        Ok(())
    }

    fn pay_closure_casts(
        &mut self,
        node: NodeId,
        growth: Growth,
        entries: usize,
    ) -> Result<(), Error> {
        let work = growth.work.saturating_add(entries);
        self.state
            .infer
            .account_work(work)
            .map_err(|e| self.error(node, e))?;
        if self.facts.source_bindings.is_some() {
            crate::sema::header_attributes::charge_source_retention(
                self.facts,
                work,
                growth.bytes,
            )?;
        }
        Ok(())
    }

    pub(super) fn hint_pointer_cast(
        &mut self,
        node: NodeId,
        target: NodeId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let Some(expected) = expected else {
            return Ok(());
        };
        let target = self.term(target)?;
        match (self.head(node, target)?, self.head(node, expected)?) {
            (None, _) => self.callable_shape(node, target, expected),
            (Some(Constructor::Pointer { .. }), Some(Constructor::Pointer { .. })) => {
                let pointee = self.child(node, target, 0)?;
                if self.head(node, pointee)?.is_none() {
                    let expected = self.child(node, expected, 0)?;
                    self.callable_shape(node, pointee, expected)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub(super) fn prepare_pointer_cast(
        &mut self,
        node: NodeId,
        value: NodeId,
        source: TermId,
        target: TermId,
    ) -> Result<(), Error> {
        let from = self.head(node, source)?;
        let to = self.head(node, target)?;
        if matches!((from, to),
            (Some(Constructor::Adt(a, _) | Constructor::AdtArgs(a, _)),
             Some(Constructor::Adt(b, _) | Constructor::AdtArgs(b, _))) if a == b)
            && self.source_nominal_coercion(value, source, target)?
        {
            return Ok(());
        }
        if matches!(from, Some(Constructor::Reference { .. }))
            && matches!(to, Some(Constructor::Reference { .. }))
        {
            let referent = self.child(node, source, 0)?;
            let pointee = self.child(node, target, 0)?;
            let heads = (self.head(node, referent)?, self.head(node, pointee)?);
            let array = matches!(
                heads,
                (
                    Some(
                        Constructor::Array(_)
                            | Constructor::ArrayTerm(_)
                            | Constructor::ArrayInfer(_)
                    ),
                    Some(Constructor::Slice)
                )
            );
            let nominal = matches!(heads,
                (Some(Constructor::Adt(a, _) | Constructor::AdtArgs(a, _)),
                 Some(Constructor::Adt(b, _) | Constructor::AdtArgs(b, _))) if a == b);
            // A reference cast is valid only as a coercion cast: to an object,
            // to a parameter that its referent unsizes to, as alloc's
            // `&value as &Dyn` under `T: Unsize<Dyn>`, or to its own referent.
            let unsizing = heads.0.is_some()
                && matches!(
                    heads.1,
                    Some(Constructor::Parameter(_) | Constructor::Dynamic { .. })
                );
            let identity = heads.0.is_some()
                && heads.0 == heads.1
                && !self
                    .state
                    .infer
                    .incompatible(&self.facts.types, referent, pointee)
                    .map_err(|e| self.error(node, e))?;
            if array || nominal || unsizing || identity {
                // The existing selection owns the actual tail and reborrow proof.
                return self.coerce(value, target);
            }
        }
        if matches!(from, Some(Constructor::Pointer { .. }))
            || matches!(to, Some(Constructor::Pointer { .. }))
        {
            self.state.pointer_casts.push(node);
        }
        if matches!(
            from,
            Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..))
        ) && (matches!(to, Some(Constructor::Pointer { .. }))
            || matches!(to, Some(Constructor::Primitive(p)) if p.integer().is_some()))
        {
            let signature = self.child(node, source, 0)?;
            return self.coerce(value, signature);
        }
        if !matches!(to, Some(Constructor::Pointer { .. })) {
            return Ok(());
        }
        if matches!(from, Some(Constructor::Reference { .. })) {
            let referent = self.child(node, source, 0)?;
            let pointee = self.child(node, target, 0)?;
            let array = matches!(
                self.head(node, referent)?,
                Some(
                    Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_)
                )
            );
            if array
                && self.head(node, pointee)? != Some(Constructor::Slice)
                && self
                    .state
                    .infer
                    .incompatible(&self.facts.types, referent, pointee)
                    .map_err(|e| self.error(node, e))?
            {
                let element = self.child(node, referent, 0)?;
                self.apply_pointer_coercion(value, source, target, element, None)?;
            } else {
                self.pointer_coercion(value, source, target)?;
            }
            return self.finish_reference_coercions();
        }
        if matches!(from, Some(Constructor::Pointer { .. })) {
            let referent = self.child(node, source, 0)?;
            let pointee = self.child(node, target, 0)?;
            if let Some(unsizing) = self.unsizing_referent(node, referent, pointee)? {
                let view = unsizing.view;
                self.apply_pointer_coercion(value, source, target, view, Some(unsizing))?;
                self.finish_reference_coercions()?;
            }
        }
        Ok(())
    }
    /// Complete the pointee of an otherwise unconstrained raw pointer cast.
    /// Body equations and explicit expected pointer hints have already run.
    pub(super) fn resolve_raw_cast_pointees(&mut self) -> Result<(), Error> {
        let body = self.state.body.node;
        self.state
            .infer
            .account_work(self.state.pointer_casts.len())
            .map_err(|error| self.error(body, error))?;
        for index in 0..self.state.pointer_casts.len() {
            let node = self.state.pointer_casts[index];
            let Kind::Cast { value, ty } = self.kind(node) else {
                return Err(self.error(node, "missing pointer cast syntax"));
            };
            let source = self.term(value)?;
            let target = self.term(ty)?;
            let (
                Some(Constructor::Pointer { mutable: from }),
                Some(Constructor::Pointer { mutable: to }),
            ) = (
                self.contextual_head(node, source)?,
                self.contextual_head(node, target)?,
            )
            else {
                continue;
            };
            if to && !from {
                continue;
            }
            let target = self.child(node, target, 0)?;
            if self
                .state
                .infer
                .unresolved_domain(target)
                .map_err(|error| self.error(node, error))?
                != Some(Domain::Any)
            {
                continue;
            }
            let pointee = self.child(node, source, 0)?;
            match self.state.infer.resolved(&mut self.facts.types, pointee) {
                Ok(_) => self
                    .state
                    .infer
                    .equate(&self.facts.types, target, pointee)
                    .map_err(|error| self.error(node, error))?,
                Err(InferError::Unresolved | InferError::Normalization) => {}
                Err(error) => return Err(self.error(node, error)),
            }
        }
        Ok(())
    }

    /// Early constant bodies suspend until these metadata roots have real fields.
    pub(super) fn pointer_cast_storage(&self, solved: &Solved) -> Result<Vec<TypeId>, Error> {
        let mut roots = Vec::new();
        for &node in &self.state.pointer_casts {
            let Kind::Cast { value, ty } = self.kind(node) else {
                return Err(self.error(node, "missing pointer cast syntax"));
            };
            for input in [value, ty] {
                let ty = solved.types[self.term(input)?.0 as usize];
                if let Some(
                    Type::Pointer { pointee, .. }
                    | Type::Reference {
                        referent: pointee, ..
                    },
                ) = self.facts.types.kind(ty)
                    && !(self.facts.types.nominal_sizes_sealed()
                        && self.facts.types.is_sized(pointee) == Some(true))
                {
                    roots.push(pointee);
                }
            }
        }
        Ok(roots)
    }
    fn cast_metadata(&mut self, node: NodeId, ty: TypeId) -> Result<Metadata, Error> {
        self.facts
            .layouts
            .pointer_metadata(&self.facts.types, &self.facts.aggregates, ty)
            .map_err(|e| self.error(node, e))
    }
    pub(super) fn nominal_cast_operation(
        &mut self,
        value: NodeId,
        target: NodeId,
    ) -> Result<Option<Operation>, Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(None);
        }
        let key = self.key(value);
        let Some(adjustment) = self.facts.node(key).and_then(|fact| fact.adjustment) else {
            return Ok(None);
        };
        if adjustment.kind != AdjustmentKind::Unsize {
            return Ok(None);
        }
        let Some(crate::types::unsizing::Conversion::Nominal {
            source: from,
            target: to,
        }) = self.facts.unsizing_adjustments.get(&key).copied()
        else {
            return Ok(None);
        };
        let source = self
            .facts
            .node(key)
            .and_then(|fact| fact.ty)
            .ok_or_else(|| self.error(value, "missing unadjusted nominal cast operand type"))?;
        let target = self.ty(target)?;
        if adjustment.target != target
            || (from, to) != (source, target)
            || !matches!((self.facts.types.kind(source), self.facts.types.kind(target)),
                (Some(Type::Adt(a, _)), Some(Type::Adt(b, _))) if a == b)
        {
            return Err(self.error(
                value,
                "coercion cast changed its selected nominal endpoints",
            ));
        }
        Ok(Some(Operation::CoercionCast { source, target }))
    }
    pub(super) fn validate_pointer_cast(
        &mut self,
        node: NodeId,
        value: NodeId,
        target: NodeId,
    ) -> Result<Option<Cast>, Error> {
        let original = self
            .facts
            .node(self.key(value))
            .and_then(|fact| fact.ty)
            .ok_or_else(|| self.error(node, "missing unadjusted cast operand type"))?;
        let source = self
            .facts
            .node(self.key(value))
            .and_then(|f| f.adjustment)
            .map_or(original, |adjustment| adjustment.target);
        let target = self.ty(target)?;
        if source == target
            && (self.reference_unsizing_cast(value, target)?
                || original == target
                    && matches!(self.facts.types.kind(target), Some(Type::Reference { .. })))
        {
            return Ok(Some(Cast::Pointer));
        }
        let (from, to) = (self.facts.types.kind(source), self.facts.types.kind(target));
        let from = if let Some(Type::Pattern { base, .. }) = from {
            self.facts.types.kind(base)
        } else {
            from
        };
        let kind = match (from, to) {
            (
                Some(Type::Pointer { pointee: left, .. }),
                Some(Type::Pointer { pointee: right, .. }),
            ) => {
                if source == target {
                    return Ok(Some(Cast::Pointer));
                }
                let left = self.cast_metadata(node, left)?;
                let right = self.cast_metadata(node, right)?;
                if right == Metadata::Thin {
                    if left == Metadata::Thin {
                        Cast::Pointer
                    } else {
                        Cast::PointerThin
                    }
                } else if left == right {
                    Cast::Pointer
                } else {
                    return Err(self.error(node, "pointer cast cannot invent or change metadata"));
                }
            }
            (Some(Type::Primitive(p)), Some(Type::Pointer { pointee, .. }))
                if p.integer().is_some() =>
            {
                if self.cast_metadata(node, pointee)? != Metadata::Thin {
                    return Err(self.error(node, "integer cast requires a thin pointer target"));
                }
                Cast::IntegerToPointer
            }
            (Some(Type::Pointer { pointee, .. }), Some(Type::Primitive(p)))
                if p.integer().is_some() =>
            {
                if self.cast_metadata(node, pointee)? != Metadata::Thin {
                    return Err(self.error(node, "pointer-to-integer cast requires a thin pointer"));
                }
                Cast::PointerToInteger
            }
            (Some(Type::Function { .. }), Some(Type::Pointer { pointee, .. })) => {
                if self.cast_metadata(node, pointee)? != Metadata::Thin {
                    return Err(self.error(node, "function cast requires a thin pointer target"));
                }
                Cast::Pointer
            }
            (Some(Type::Function { .. }), Some(Type::Primitive(p))) if p.integer().is_some() => {
                Cast::PointerToInteger
            }
            _ => return Ok(None),
        };
        if kind == Cast::PointerToInteger && !self.state.in_function {
            return Err(self.error(
                node,
                "pointers cannot be cast to integers during const eval",
            ));
        }
        Ok(Some(kind))
    }
    fn reference_unsizing_cast(
        &mut self,
        mut value: NodeId,
        target: TypeId,
    ) -> Result<bool, Error> {
        while let Kind::Group(child) | Kind::Attributed { value: child, .. } = self.kind(value) {
            self.state.reference_steps += 1;
            if self.state.reference_steps > 1_000_000 {
                return Err(self.error(value, "reference cast wrapper work budget exceeded"));
            }
            value = child;
        }
        Ok(selected_reference_cast(self.facts, self.key(value), target))
    }
}
/// A reference cast is valid only through the child's own selected
/// coercion, which each shape checks against its exact record.
fn selected_reference_cast(
    facts: &crate::sema::model::Declarations,
    source: crate::resolve::unit::NodeKey,
    target: TypeId,
) -> bool {
    let Some(fact) = facts.node(source) else {
        return false;
    };
    let Some(adjustment) = fact.adjustment else {
        return false;
    };
    if adjustment.target != target {
        return false;
    }
    let (
        Some(Type::Reference {
            referent: from,
            mutable: from_mutable,
            ..
        }),
        Some(Type::Reference {
            referent: to,
            mutable: to_mutable,
            ..
        }),
    ) = (
        fact.ty.and_then(|ty| facts.types.kind(ty)),
        facts.types.kind(target),
    )
    else {
        return false;
    };
    if to_mutable && !from_mutable {
        return false;
    }
    let terminal = facts.unsizing_adjustments.get(&source).copied();
    match adjustment.kind {
        AdjustmentKind::ArrayToSlice | AdjustmentKind::Unsize | AdjustmentKind::Reborrow => {
            match (facts.types.kind(from), facts.types.kind(to)) {
                (Some(Type::Array { .. } | Type::ArrayTerm { .. }), Some(Type::Slice(_))) => true,
                (Some(Type::Adt(a, _)), Some(Type::Adt(b, _))) if a == b && from != to => {
                    terminal.is_some()
                }
                (_, Some(kind @ (Type::Dynamic { .. } | Type::Parameter(_)))) if from != to => {
                    adjustment.kind != AdjustmentKind::ArrayToSlice
                        && terminal.is_some_and(|terminal| {
                            selected_referent_pair(terminal, kind, from, to)
                        })
                }
                // Only a plain reborrow keeps its referent; an unsizing never does.
                _ => {
                    from == to && adjustment.kind == AdjustmentKind::Reborrow && terminal.is_none()
                }
            }
        }
        // The same referent, with only a directed lifetime relation retained.
        AdjustmentKind::Equality => from == to && terminal.is_none(),
        _ => false,
    }
}
/// Whether the selected terminal conversion relates exactly these referents:
/// an object or upcast for an object target, and a clause-proved bound
/// unsizing for a type parameter target.
fn selected_referent_pair(
    terminal: crate::types::unsizing::Conversion<TypeId>,
    target_kind: Type,
    from: TypeId,
    to: TypeId,
) -> bool {
    use crate::types::unsizing::Conversion;
    let pair = match (terminal, target_kind) {
        (
            Conversion::Object {
                concrete: source,
                target,
            }
            | Conversion::Upcast { source, target },
            Type::Dynamic { .. },
        )
        | (Conversion::Bound { source, target }, Type::Parameter(_)) => (source, target),
        _ => return false,
    };
    pair == (from, to)
}
#[cfg(test)]
#[path = "../../../tests/invariants/unsizing_casts.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/invariants/postponed_casts.rs"]
mod postponed_cast_tests;
