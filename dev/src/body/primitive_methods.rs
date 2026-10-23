//! Validated primitive selections materialize ordinary typed and independently proved CFGs.
mod forwarded;
mod ordering;
mod sequences;
use super::{
    ir::{
        Action, Body, BodyKey, Local, LocalId, LocalKind, Owner, Place, Projection, ProjectionId,
        ProjectionKind, Rvalue, Statement, Value, ValueId,
    },
    regions::Contract,
};
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::{
        model::Declarations,
        primitive_methods::{Compound, Form, Rhs, Selected, Validated},
    },
    types::{Kind, TypeId, substitute::Bound},
};

/// Only independently validated factory bodies may omit a source template.
#[derive(Clone, Copy)]
pub(crate) struct Permit {
    instance: crate::types::instance::Instance,
}
impl Permit {
    pub(crate) const fn instance(self) -> crate::types::instance::Instance {
        self.instance
    }
}

pub(crate) fn build(
    report: &Report,
    facts: &mut Declarations,
    selected: Selected,
) -> Result<(Body, Contract), Error> {
    let source = selected
        .validate(facts)
        .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
    let error = |e| report.body_error(source.source, e);
    if matches!(facts.types.kind(source.result), Some(Kind::Adt(..))) {
        facts
            .aggregates
            .instantiate(&mut facts.types, source.result)
            .map_err(error)?;
    }
    let (mut contract, parameters) = if matches!(source.compound, Compound::None) {
        let (contract, right) =
            contract(facts, &source, selected.descriptor().form()).map_err(error)?;
        (contract, [source.receiver, right])
    } else {
        forwarded::contract(facts, selected, &source).map_err(error)?
    };
    let mut body = Body::new(
        Owner::Function {
            definition: source.definition,
            signature: source.signature,
        },
        source.source,
        source.result,
    );
    body.instance = Some(selected.instance());
    body.storage = Some(super::storage::Facts::new(super::scopes::Facts::synthetic(
        source.source.node,
    )));
    for (index, ty) in parameters.into_iter().enumerate() {
        let ordinal = u32::try_from(index).map_err(|_| error("too many factory parameters"))?;
        body.locals.push(Local {
            scope: super::scopes::Id(0),
            storage_parent: ordinal.checked_sub(1).map(LocalId),
            ty,
            kind: LocalKind::Parameter(ordinal),
            definition: None,
            mutable: false,
            source: source.source.node.into(),
        });
        statement(&mut body, Action::StorageLive(LocalId(ordinal)));
    }
    let result = if matches!(source.compound, Compound::Sequences(_)) {
        sequences::operation(&mut body, facts, &source, parameters).map_err(error)?;
        None
    } else if !matches!(source.compound, Compound::None) {
        Some(forwarded::operation(&mut body, facts, &source, parameters).map_err(error)?)
    } else {
        operation(&mut body, facts, selected, &source, parameters[1]).map_err(error)?
    };
    if let Some(result) = result {
        super::build::finish_factory(&mut body, result).map_err(error)?;
    }
    if !matches!(source.compound, Compound::None) {
        super::dispatch::resolve(facts, &mut body, &mut contract)
            .map_err(|e| report.body_error(source.source, e))?;
    }
    validate(facts, selected, &body, &contract).map_err(error)?;
    Ok((body, contract))
}
fn operation(
    body: &mut Body,
    facts: &Declarations,
    selected: Selected,
    source: &Validated,
    right: TypeId,
) -> Result<Option<ValueId>, &'static str> {
    let types = &facts.types;
    let descriptor = selected.descriptor();
    let scalar = descriptor.scalars()?;
    let left = if descriptor.form() == Form::ValueBinary {
        Place::Local(LocalId(0))
    } else {
        dereference(body, Place::Local(LocalId(0)), source.receiver, types)?
    };
    let a = value(
        body,
        descriptor.operand_type(facts, scalar.left)?,
        Rvalue::Read(left),
    )?;
    let right = if scalar.rhs == Rhs::SharedReference || descriptor.form() == Form::Comparison {
        dereference(body, Place::Local(LocalId(1)), right, types)?
    } else {
        Place::Local(LocalId(1))
    };
    let b = value(
        body,
        descriptor.operand_type(facts, scalar.right)?,
        Rvalue::Read(right),
    )?;
    let (a, b) = if descriptor.nominal() {
        let ty = TypeId::primitive(crate::types::Primitive::I8);
        let a = value(
            body,
            ty,
            Rvalue::Cast {
                direct_variant: false,
                kind: crate::sema::model::Cast::EnumToInteger,
                operand: a,
            },
        )?;
        let b = value(
            body,
            ty,
            Rvalue::Cast {
                direct_variant: false,
                kind: crate::sema::model::Cast::EnumToInteger,
                operand: b,
            },
        )?;
        (a, b)
    } else {
        (a, b)
    };
    if matches!(
        source.comparison,
        Some(
            crate::support::comparisons::Method::PartialCmp
                | crate::support::comparisons::Method::Cmp
        )
    ) {
        ordering::build(body, facts, source, a, b)?;
        return Ok(None);
    }
    let result = value(
        body,
        if descriptor.form() == Form::Comparison {
            source.result
        } else {
            TypeId::primitive(scalar.left)
        },
        Rvalue::Binary {
            operands: crate::body::ir::BinaryOperands::Neither,
            operator: source.operator,
            left: a,
            right: b,
        },
    )?;
    let result = if descriptor.form() == Form::Assignment {
        statement(
            body,
            Action::Store {
                place: left,
                value: result,
            },
        );
        value(body, TypeId::UNIT, Rvalue::Unit)?
    } else {
        result
    };
    Ok(Some(result))
}
fn contract(
    facts: &mut Declarations,
    source: &Validated,
    form: Form,
) -> Result<(Contract, TypeId), &'static str> {
    // The receiver keeps its own late slot. RHS argument lifetimes belong to
    // the caller and become distinct universals in this factory's body proof.
    let Some(Kind::Function {
        bound_regions: base,
        ..
    }) = facts.types.kind(source.signature)
    else {
        return Err("primitive method has no callable signature");
    };
    let mut regions = Bound::universals(base);
    let right = if form == Form::Comparison {
        source.right
    } else {
        regions.apply(&mut facts.types, source.right)?
    };
    let universals = base
        .checked_add(
            u32::try_from(regions.universal_count()).map_err(|_| "too many factory lifetimes")?,
        )
        .ok_or("too many factory lifetimes")?;
    let parameters = facts.types.intern_list(&[source.receiver, right])?;
    let signature = facts.types.intern(Kind::Function {
        parameters,
        result: source.result,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
        bound_regions: universals,
    })?;
    let mut contract = super::regions::callable(
        facts,
        source.definition,
        source.definition,
        source.source,
        signature,
    )?;
    contract.parameters.clear();
    Ok((contract, right))
}
fn statement(body: &mut Body, action: Action) {
    body.statements.push(Statement {
        source: body.root_site(),
        action,
    });
}
fn value(body: &mut Body, ty: TypeId, kind: Rvalue) -> Result<ValueId, &'static str> {
    let id = ValueId(u32::try_from(body.values.len()).map_err(|_| "too many factory values")?);
    body.values.push(Value { ty, kind });
    statement(body, Action::Evaluate(id));
    Ok(id)
}
fn dereference(
    body: &mut Body,
    base: Place,
    ty: TypeId,
    types: &crate::types::Store,
) -> Result<Place, &'static str> {
    let Some(Kind::Reference {
        referent,
        mutable,
        region,
    }) = types.kind(ty)
    else {
        return Err("factory dereference requires a reference");
    };
    let id = ProjectionId(
        u32::try_from(body.projections.len()).map_err(|_| "too many factory projections")?,
    );
    body.projections.push(Projection {
        base,
        kind: ProjectionKind::Dereference { region, mutable },
        ty: referent,
    });
    statement(body, Action::Project(id));
    Ok(Place::Projection(id))
}
pub(crate) fn validate(
    facts: &mut Declarations,
    selected: Selected,
    body: &Body,
    input: &Contract,
) -> Result<Permit, &'static str> {
    let source = selected.validate(facts)?;
    if body.instance != Some(selected.instance())
        || body.key() != BodyKey::Declaration(source.definition)
        || input.key != body.key()
        || input.source != body.source
        || body.source != source.source.into()
        || !matches!(body.owner,Owner::Function{definition,signature} if definition==source.definition && signature==source.signature)
        || body.result != source.result
        || !input.parameters.is_empty()
    {
        return Err("primitive factory body or contract differs from selected instance");
    }
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    }) = input.signature.and_then(|ty| facts.types.kind(ty))
    else {
        return Err("primitive factory contract has no valid signature");
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("missing factory contract parameters")?
        .to_vec();
    if result != source.result || bound_regions != input.universals || parameters.len() != 2 {
        return Err("primitive factory parameters differ from lifetime contract");
    }
    validate_locals(body, &source, &parameters)?;
    if !matches!(source.compound, Compound::None) {
        forwarded::validate(facts, selected, &source, input, &parameters)?;
        return Ok(Permit {
            instance: selected.instance(),
        });
    }
    let base = match selected.descriptor().form() {
        Form::Assignment => 1,
        Form::ValueBinary => 0,
        Form::Comparison => 2,
    };
    let scalar = selected.descriptor().scalars()?;
    let (right, count) = match scalar.rhs {
        Rhs::Owned => (source.right, base),
        Rhs::SharedReference => {
            let Some(Kind::Reference { region, .. }) = facts.types.kind(source.right) else {
                return Err("missing selected reference RHS");
            };
            let expected = if region == crate::types::RegionId::STATIC {
                region
            } else {
                crate::types::RegionId::bound(
                    0,
                    u16::try_from(base).map_err(|_| "invalid factory lifetime base")?,
                )?
            };
            let Some(Kind::Reference {
                referent,
                mutable: false,
                region,
            }) = facts.types.kind(parameters[1])
            else {
                return Err("factory RHS lost shared reference identity");
            };
            if region != expected || referent != TypeId::primitive(scalar.right) {
                return Err("factory RHS lifetime differs from selected argument abstraction");
            }
            (
                parameters[1],
                if expected == crate::types::RegionId::STATIC {
                    base
                } else {
                    base + 1
                },
            )
        }
    };
    if parameters != [source.receiver, right]
        || input.universals != count
        || !input.assumptions.is_empty()
        || !input.type_assumptions.is_empty()
        || !input.obligations.is_empty()
        || !input.proofs.is_empty()
        || input.builtins.is_some()
    {
        return Err("primitive factory contract differs from validated lifetime abstraction");
    }
    Ok(Permit {
        instance: selected.instance(),
    })
}

fn validate_locals(
    body: &Body,
    source: &Validated,
    parameters: &[TypeId],
) -> Result<(), &'static str> {
    let sequence = matches!(source.compound, Compound::Sequences(_));
    let expected = if sequence {
        if source.comparison == Some(crate::support::comparisons::Method::Eq) {
            3
        } else {
            4
        }
    } else {
        2
    };
    if body.locals.len() != expected
        || body
            .locals
            .iter()
            .zip(parameters)
            .any(|(local, &ty)| local.ty != ty)
    {
        return Err("primitive factory locals differ from lifetime contract");
    }
    if sequence {
        sequences::validate(body, source)?;
    }
    Ok(())
}
