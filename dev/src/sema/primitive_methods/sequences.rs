//! Sequence evidence retains exact container shapes and the element blanket condition.
use super::{
    Candidate, Compound, Context, Declarations, Descriptor, Operands, Operation, Predicate,
    Sequence, infer,
};
use crate::{
    support::comparisons::Comparison,
    types::{
        Kind, TypeId,
        infer::{Constructor, TermId, arguments::Argument},
    },
};

fn head(
    facts: &Declarations,
    context: &mut Context,
    term: TermId,
) -> Result<Option<(Sequence, TermId, Option<Constructor>)>, infer::Error> {
    let Some(head) = context.head(&facts.types, term)? else {
        return Ok(None);
    };
    let (shape, sequence, head) = match head {
        Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_) => {
            (Sequence::Array, term, head)
        }
        Constructor::Slice => (Sequence::Slice, term, head),
        Constructor::Reference { mutable, .. } => {
            let sequence = context.child(&facts.types, term, 0)?;
            if context.head(&facts.types, sequence)? != Some(Constructor::Slice) {
                return Ok(None);
            }
            (
                if mutable {
                    Sequence::MutableSlice
                } else {
                    Sequence::SharedSlice
                },
                sequence,
                Constructor::Slice,
            )
        }
        _ => return Ok(None),
    };
    Ok(Some((
        shape,
        context.child(&facts.types, sequence, 0)?,
        (shape == Sequence::Array).then_some(head),
    )))
}
fn compatible(kind: Comparison, left: Sequence, right: Sequence) -> bool {
    match kind {
        Comparison::Eq => left == right && matches!(left, Sequence::Array | Sequence::Slice),
        Comparison::PartialEq => matches!(
            (left, right),
            (
                Sequence::Array,
                Sequence::Array | Sequence::Slice | Sequence::SharedSlice | Sequence::MutableSlice
            ) | (
                Sequence::Slice | Sequence::SharedSlice | Sequence::MutableSlice,
                Sequence::Array
            ) | (Sequence::Slice, Sequence::Slice)
        ),
        Comparison::PartialOrd | Comparison::Ord => {
            left == right && matches!(left, Sequence::Array | Sequence::Slice)
        }
    }
}
pub(super) fn candidate(
    facts: &Declarations,
    context: &mut Context,
    goal: &Predicate,
    right: TermId,
    kind: Comparison,
) -> Result<Option<Candidate>, infer::Error> {
    let left = head(facts, context, goal.subject())?;
    let right_shape = head(facts, context, right)?;
    let (Some((left, a, n)), Some((right, b, m))) = (left, right_shape) else {
        // Applicability precedes ordinary argument checking. An unresolved RHS
        // cannot exclude a known sequence candidate; no shape is inferred here.
        let unknown = left.is_some() && context.head(&facts.types, right)?.is_none()
            || right_shape.is_some() && context.head(&facts.types, goal.subject())?.is_none();
        return Ok(unknown.then_some(Candidate::Deferred));
    };
    if !compatible(kind, left, right) {
        return Ok(Some(Candidate::Unrelated));
    }
    if let (Some(n), Some(m)) = (n, m) {
        let n = context.array_length(&facts.types, n)?;
        let m = context.array_length(&facts.types, m)?;
        if let Err(error) = context.equate_consts(n, m) {
            return if error == infer::Error::Mismatch {
                Ok(Some(Candidate::Unrelated))
            } else {
                Err(error)
            };
        }
    }
    if matches!(kind, Comparison::PartialOrd | Comparison::Ord)
        && let Err(error) =
            context.equate_input_pairs(&facts.types, &[(Argument::Type(a), Argument::Type(b))])
    {
        return if error == infer::Error::Mismatch {
            Ok(Some(Candidate::Unrelated))
        } else {
            Err(error)
        };
    }
    let (item, _) = goal
        .trait_arguments()
        .ok_or(infer::Error::InvalidIdentity)?;
    let arguments = if kind.has_rhs() {
        vec![Argument::Type(b)]
    } else {
        Vec::new()
    };
    Ok(Some(Candidate::Conditional {
        descriptor: Descriptor {
            operation: Operation::Sequences(kind),
            operands: Operands::Sequences { left, right },
        },
        prerequisite: Predicate::trait_application(a, item, arguments),
    }))
}

pub(super) fn element(
    facts: &Declarations,
    ty: TypeId,
    shape: Sequence,
) -> Result<TypeId, &'static str> {
    let ty = match (shape, facts.types.kind(ty)) {
        (
            Sequence::SharedSlice,
            Some(Kind::Reference {
                referent,
                mutable: false,
                ..
            }),
        )
        | (
            Sequence::MutableSlice,
            Some(Kind::Reference {
                referent,
                mutable: true,
                ..
            }),
        ) => referent,
        (Sequence::Array | Sequence::Slice, _) => ty,
        _ => return Err("sequence reference shape differs from descriptor"),
    };
    match (shape, facts.types.kind(ty)) {
        (Sequence::Array, Some(Kind::Array { element, .. } | Kind::ArrayTerm { element, .. }))
        | (
            Sequence::Slice | Sequence::SharedSlice | Sequence::MutableSlice,
            Some(Kind::Slice(element)),
        ) => Ok(element),
        _ => Err("sequence container differs from descriptor"),
    }
}
pub(super) fn validate(
    facts: &Declarations,
    descriptor: Descriptor,
    kind: Comparison,
    operands: [TypeId; 2],
    parameters: [TypeId; 2],
    bound_regions: u32,
) -> Result<(), &'static str> {
    let Operands::Sequences { left, right } = descriptor.operands else {
        return Err("sequence operation requires sequence operands");
    };
    if kind == Comparison::Eq || bound_regions != 2 || !compatible(kind, left, right) {
        return Err("sequence member has invalid blanket contract");
    }
    let a = element(facts, operands[0], left)?;
    let b = element(facts, operands[1], right)?;
    if matches!(kind, Comparison::PartialOrd | Comparison::Ord) {
        element_equalities(facts, a, b)?;
    }
    // The canonical substitution check validates each full referent beneath
    // this callable binder; raw operands are outside that binder.
    for (slot, parameter) in parameters.into_iter().enumerate() {
        if !matches!(facts.types.kind(parameter),Some(Kind::Reference{mutable:false,region,..}) if region==crate::types::RegionId::bound(0,u16::try_from(slot).map_err(|_|"invalid sequence slot")?)?)
        {
            return Err("sequence comparison requires independent shared borrows");
        }
    }
    if left == Sequence::Array && right == Sequence::Array {
        match (facts.types.kind(operands[0]), facts.types.kind(operands[1])) {
            (Some(Kind::Array { length: a, .. }), Some(Kind::Array { length: b, .. }))
                if a == b => {}
            (Some(Kind::ArrayTerm { length: a, .. }), Some(Kind::ArrayTerm { length: b, .. }))
                if a == b => {}
            _ => return Err("sequence descriptor requires equal canonical array lengths"),
        }
    }
    Ok(())
}
/// These are exactly the canonical blanket impl's one-T unification equations,
/// not facts inferred from a result or a source-local condition. Selection pays
/// the same input equations before this factory's ordinary body can be demanded.
fn element_equalities(
    facts: &Declarations,
    left: TypeId,
    right: TypeId,
) -> Result<Vec<crate::types::outlives::Pair>, &'static str> {
    if left == right {
        return Ok(Vec::new());
    }
    let mut context = Context::default();
    let a = context
        .known(&facts.types, left)
        .map_err(|_| "invalid sequence element")?;
    let b = context
        .known(&facts.types, right)
        .map_err(|_| "invalid sequence element")?;
    context
        .equate_input_pairs(&facts.types, &[(Argument::Type(a), Argument::Type(b))])
        .map_err(|_| "sequence ordering elements do not match one canonical type")?;
    Ok(context.region_equalities)
}
impl super::Selected {
    pub(crate) fn sequence_inputs(
        self,
        facts: &mut Declarations,
    ) -> Result<Option<super::referential::Inputs>, &'static str> {
        let source = self.validate(facts)?;
        let Compound::Sequences(operands) = source.compound else {
            return Ok(None);
        };
        if facts.types.contains_parameters(self.instance().ty()) != Some(false) {
            return Err("sequence factory requires a complete concrete instance");
        }
        let Operands::Sequences { left, right } = self.descriptor.operands else {
            return Err("missing sequence descriptor");
        };
        let mut abstraction = crate::types::substitute::Bound::universals(2);
        let operands = [
            abstraction.apply(&mut facts.types, operands[0])?,
            abstraction.apply(&mut facts.types, operands[1])?,
        ];
        let universals = 2u32
            .checked_add(
                u32::try_from(abstraction.universal_count())
                    .map_err(|_| "too many sequence lifetimes")?,
            )
            .ok_or("too many sequence lifetimes")?;
        let a = element(facts, operands[0], left)?;
        let b = element(facts, operands[1], right)?;
        let owner = facts
            .associated
            .trait_owner(source.definition)
            .ok_or("sequence method has no trait owner")?;
        let arguments = if source.comparison == Some(crate::support::comparisons::Method::Cmp) {
            Vec::new()
        } else {
            vec![crate::types::arguments::Argument::Type(b)]
        };
        let args = facts.types.intern_arguments(&arguments)?;
        let application =
            facts
                .types
                .intern_trait_application(crate::types::traits::Application {
                    item: crate::types::ItemId(owner.0),
                    arguments: args,
                })?;
        let prerequisite = crate::types::parameters::predicates::Predicate {
            ty: a,
            bound: crate::types::parameters::predicates::Bound::Trait(application),
        };
        let equalities = if source.comparison == Some(crate::support::comparisons::Method::Eq) {
            Vec::new()
        } else {
            element_equalities(facts, a, b)?
        };
        let mut assumptions = crate::sema::callee_premises::declared_condition(
            facts,
            prerequisite.into(),
            universals,
        )?;
        for (left, right) in equalities {
            assumptions.regions.extend([(left, right), (right, left)]);
        }
        let assumptions = Some(assumptions);
        Ok(Some(super::referential::Inputs {
            operands,
            universals,
            assumptions,
        }))
    }
}
