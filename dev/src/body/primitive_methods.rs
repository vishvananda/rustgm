//! Validated primitive selections materialize ordinary typed and independently proved CFGs.
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
        primitive_methods::{Form, Rhs, Selected, Validated},
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
    let error = |e| Declarations::error(report, source.source, e);
    let (contract, right) = contract(facts, &source).map_err(error)?;
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
    for (index, ty) in [source.receiver, right].into_iter().enumerate() {
        let ordinal = u32::try_from(index).map_err(|_| error("too many factory parameters"))?;
        body.locals.push(Local {
            scope: super::scopes::Id(0),
            storage_parent: ordinal.checked_sub(1).map(LocalId),
            ty,
            kind: LocalKind::Parameter(ordinal),
            definition: None,
            mutable: false,
            source: source.source.node,
        });
        statement(&mut body, Action::StorageLive(LocalId(ordinal)));
    }
    let result = operation(&mut body, &facts.types, selected, &source, right).map_err(error)?;
    super::build::finish_factory(&mut body, result).map_err(error)?;
    validate(facts, selected, &body, &contract).map_err(error)?;
    Ok((body, contract))
}
fn operation(
    body: &mut Body,
    types: &crate::types::Store,
    selected: Selected,
    source: &Validated,
    right: TypeId,
) -> Result<ValueId, &'static str> {
    let descriptor = selected.descriptor();
    let left = if descriptor.form() == Form::Assignment {
        dereference(body, Place::Local(LocalId(0)), source.receiver, types)?
    } else {
        Place::Local(LocalId(0))
    };
    let a = value(
        body,
        TypeId::primitive(descriptor.left()),
        Rvalue::Read(left),
    )?;
    let right = if descriptor.rhs() == Rhs::SharedReference {
        dereference(body, Place::Local(LocalId(1)), right, types)?
    } else {
        Place::Local(LocalId(1))
    };
    let b = value(
        body,
        TypeId::primitive(descriptor.right()),
        Rvalue::Read(right),
    )?;
    let result = value(
        body,
        TypeId::primitive(descriptor.left()),
        Rvalue::Binary {
            operator: descriptor.operator()?,
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
    Ok(result)
}
fn contract(
    facts: &mut Declarations,
    source: &Validated,
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
    let right = regions.apply(&mut facts.types, source.right)?;
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
        source: body.source.node,
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
        || body.source != source.source
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
        .ok_or("missing factory contract parameters")?;
    if result != source.result
        || bound_regions != input.universals
        || body.locals.len() != 2
        || parameters.len() != 2
        || body
            .locals
            .iter()
            .zip(parameters)
            .any(|(local, &ty)| local.ty != ty)
    {
        return Err("primitive factory parameters differ from lifetime contract");
    }
    let base = u32::from(selected.descriptor().form() == Form::Assignment);
    let (right, count) = match selected.descriptor().rhs() {
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
            if region != expected || referent != TypeId::primitive(selected.descriptor().right()) {
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
