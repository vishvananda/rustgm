//! Reference blanket evidence pays the exact referent condition at each layer.
use super::{Candidate, Context, Declarations, Descriptor, Operands, Operation, Predicate, infer};
use crate::{
    support::comparisons::Comparison,
    types::infer::{Constructor, TermId, arguments::Argument},
};

pub(super) fn candidate(
    facts: &Declarations,
    context: &mut Context,
    goal: &Predicate,
    right: TermId,
    kind: Comparison,
) -> Result<Option<Candidate>, infer::Error> {
    let left_head = context.head(&facts.types, goal.subject())?;
    let right_head = context.head(&facts.types, right)?;
    let (
        Some(Constructor::Reference {
            mutable: left_mutable,
            ..
        }),
        Some(Constructor::Reference {
            mutable: right_mutable,
            ..
        }),
    ) = (left_head, right_head)
    else {
        return Ok(None);
    };
    if kind != Comparison::PartialEq && left_mutable != right_mutable {
        return Ok(Some(Candidate::Unrelated));
    }
    let (item, _) = goal
        .trait_arguments()
        .ok_or(infer::Error::InvalidIdentity)?;
    let left = context.child(&facts.types, goal.subject(), 0)?;
    let right = context.child(&facts.types, right, 0)?;
    let arguments = if matches!(kind, Comparison::PartialEq | Comparison::PartialOrd) {
        vec![Argument::Type(right)]
    } else {
        Vec::new()
    };
    Ok(Some(Candidate::Conditional {
        descriptor: Descriptor {
            operation: Operation::References(kind),
            operands: Operands::References {
                left_mutable,
                right_mutable,
            },
        },
        prerequisite: Predicate::trait_application(left, item, arguments),
    }))
}

/// The member template independently ties the shifted Self/Rhs to these
/// physical borrows; this gate verifies the exact blanket shape and permission.
pub(super) fn validate(
    facts: &Declarations,
    descriptor: Descriptor,
    kind: Comparison,
    operands: [crate::types::TypeId; 2],
    parameters: [crate::types::TypeId; 2],
    bound_regions: u32,
) -> Result<(), &'static str> {
    use crate::types::{Kind, RegionId};
    let Operands::References {
        left_mutable,
        right_mutable,
    } = descriptor.operands
    else {
        return Err("reference operation requires reference operands");
    };
    if kind == Comparison::Eq
        || bound_regions != 2
        || kind != Comparison::PartialEq && left_mutable != right_mutable
    {
        return Err("reference comparison member has invalid blanket contract");
    }
    for ((operand, parameter), (mutable, slot)) in operands
        .into_iter()
        .zip(parameters)
        .zip([(left_mutable, 0), (right_mutable, 1)])
    {
        if !matches!(facts.types.kind(operand),Some(Kind::Reference{mutable:actual,..}) if actual==mutable)
            || !matches!(facts.types.kind(parameter),Some(Kind::Reference{mutable:false,region,..}) if region==RegionId::bound(0,slot)?)
        {
            return Err("reference comparison operand differs from selected blanket shape");
        }
    }
    Ok(())
}

pub(crate) struct Inputs {
    pub operands: [crate::types::TypeId; 2],
    pub universals: u32,
    pub assumptions: Option<crate::sema::callee_premises::Assumptions>,
}
impl super::Selected {
    /// The only inherited premise is the exact declared blanket condition.
    /// Ordinary caller selection has already proved it; this reconstruction is
    /// independent of factory syntax or its generated call's normalization.
    pub(crate) fn reference_inputs(
        self,
        facts: &mut Declarations,
    ) -> Result<Option<Inputs>, &'static str> {
        let source = self.validate(facts)?;
        if facts.types.contains_parameters(self.instance().ty()) != Some(false) {
            return Err("reference factory requires a complete concrete instance");
        }
        let super::Compound::References(operands) = source.compound else {
            return Ok(None);
        };
        let Operation::References(kind) = self.descriptor.operation else {
            return Err("reference operands require the reference operation");
        };
        let mut abstraction = crate::types::substitute::Bound::universals(2);
        let operands = [
            abstraction.apply(&mut facts.types, operands[0])?,
            abstraction.apply(&mut facts.types, operands[1])?,
        ];
        let universals = 2u32
            .checked_add(
                u32::try_from(abstraction.universal_count())
                    .map_err(|_| "too many reference factory lifetimes")?,
            )
            .ok_or("too many reference factory lifetimes")?;
        let [left, right] = operands.map(|ty| match facts.types.kind(ty) {
            Some(crate::types::Kind::Reference { referent, .. }) => Ok(referent),
            _ => Err("reference factory lost operand identity"),
        });
        let (left, right) = (left?, right?);
        let owner = facts
            .associated
            .trait_owner(source.definition)
            .ok_or("reference method has no trait owner")?;
        let arguments = if kind == Comparison::Ord {
            Vec::new()
        } else {
            vec![crate::types::arguments::Argument::Type(right)]
        };
        let arguments = facts.types.intern_arguments(&arguments)?;
        let application =
            facts
                .types
                .intern_trait_application(crate::types::traits::Application {
                    item: crate::types::ItemId(owner.0),
                    arguments,
                })?;
        let prerequisite = crate::types::parameters::predicates::Predicate {
            ty: left,
            bound: crate::types::parameters::predicates::Bound::Trait(application),
        };
        let assumptions = Some(crate::sema::callee_premises::declared_condition(
            facts,
            prerequisite.into(),
            universals,
        )?);
        Ok(Some(Inputs {
            operands,
            universals,
            assumptions,
        }))
    }
}
