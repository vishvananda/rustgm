//! Provided relations call the canonical `partial_cmp` and inspect its actual constructors.
use super::{ProvidedCall, call, statement, value};
use crate::{
    body::{
        ir::{
            Action, Block, BlockId, Body, Local, LocalId, LocalKind, Owner, Place, Projection,
            ProjectionId, ProjectionKind, Rvalue, ScalarId, Terminator, ValueId, ValueList,
            VariantId,
        },
        scopes,
    },
    resolve::{model::DefId, unit::NodeKey},
    sema::{members::Selection, model::Declarations},
    support::comparisons::Method,
    types::{Kind, Primitive, TypeId},
};
pub(super) fn build(
    facts: &mut Declarations,
    definition: DefId,
    partial_cmp: DefId,
    relation: Method,
    signature: TypeId,
    source: NodeKey,
) -> Result<Body, &'static str> {
    let ProvidedCall {
        parameters,
        result,
        callee,
        signature: opened,
    } = call(facts, partial_cmp, signature)?;
    facts.aggregates.instantiate(&mut facts.types, result)?;
    if !relation.relation() {
        return Err("provided order relation has incorrect member");
    }
    let OrderShape {
        ordering,
        some,
        less,
        equal,
        greater,
        field,
    } = OrderShape::new(facts, result)?;
    let boolean = TypeId::primitive(Primitive::Bool);
    let mut body = initialize(definition, signature, source, parameters, result)?;
    let function = value(&mut body, callee, Rvalue::Function(partial_cmp))?;
    let comparison = value(
        &mut body,
        result,
        Rvalue::Call {
            signature: opened,
            callee: function,
            arguments: ValueList { start: 0, end: 2 },
        },
    )?;
    statement(
        &mut body,
        Action::Store {
            place: Place::Local(LocalId(2)),
            value: comparison,
        },
    );
    let condition = value(
        &mut body,
        boolean,
        Rvalue::IsVariant {
            place: Place::Local(LocalId(2)),
            variant: some,
        },
    )?;
    let end = u32::try_from(body.statements.len()).map_err(|_| "too many provided statements")?;
    let inclusive = matches!(relation, Method::Le | Method::Ge);
    let accepted = if inclusive { 3 } else { 2 };
    let rejected = accepted + 1;
    branch(&mut body, 0, end, condition, 1, rejected);
    let projection = ProjectionId(0);
    body.projections.push(Projection {
        base: Place::Local(LocalId(2)),
        kind: ProjectionKind::Field(Selection::Nominal(field)),
        ty: ordering,
    });
    statement(&mut body, Action::Project(projection));
    let target = if matches!(relation, Method::Lt | Method::Le) {
        less
    } else {
        greater
    };
    let first = value(
        &mut body,
        boolean,
        Rvalue::IsVariant {
            place: Place::Projection(projection),
            variant: target,
        },
    )?;
    let next = u32::try_from(body.statements.len()).map_err(|_| "too many provided statements")?;
    branch(
        &mut body,
        end,
        next,
        first,
        accepted,
        if inclusive { 2 } else { rejected },
    );
    let start = if inclusive {
        let equal = value(
            &mut body,
            boolean,
            Rvalue::IsVariant {
                place: Place::Projection(projection),
                variant: equal,
            },
        )?;
        let end =
            u32::try_from(body.statements.len()).map_err(|_| "too many provided statements")?;
        branch(&mut body, next, end, equal, accepted, rejected);
        end
    } else {
        next
    };
    boolean_return(&mut body, start, true)?;
    let start = u32::try_from(body.statements.len()).map_err(|_| "too many provided statements")?;
    boolean_return(&mut body, start, false)?;
    Ok(body)
}

fn branch(body: &mut Body, start: u32, end: u32, condition: ValueId, yes: u32, no: u32) {
    body.blocks.push(Block {
        start,
        end,
        source: body.root_site(),
        terminator: Terminator::Branch {
            condition,
            yes: BlockId(yes),
            no: BlockId(no),
        },
    });
}
fn boolean_return(body: &mut Body, start: u32, accepted: bool) -> Result<(), &'static str> {
    let scalar =
        ScalarId(u32::try_from(body.scalars.len()).map_err(|_| "too many provided scalars")?);
    body.scalars
        .push(crate::sema::model::Scalar::Boolean(accepted));
    let result = value(body, body.result, Rvalue::Scalar(scalar))?;
    crate::body::build::finish_factory_block(body, start, result)
}

struct OrderShape {
    ordering: TypeId,
    some: VariantId,
    less: VariantId,
    equal: VariantId,
    greater: VariantId,
    field: crate::sema::members::FieldId,
}
impl OrderShape {
    fn new(facts: &Declarations, result: TypeId) -> Result<Self, &'static str> {
        let Some(Kind::Adt(option, arguments)) = facts.types.kind(result) else {
            return Err("partial_cmp result is not a nominal option");
        };
        let [_, some] = facts.members.variants(option) else {
            return Err("partial_cmp option has incorrect variants");
        };
        let some = *some;
        let Some(crate::types::arguments::Argument::Type(ordering)) = facts
            .types
            .arguments(arguments)
            .and_then(|mut args| args.next())
        else {
            return Err("partial_cmp option has no ordering argument");
        };
        let Some(Kind::Adt(owner, _)) = facts.types.kind(ordering) else {
            return Err("partial_cmp payload has no ordering owner");
        };
        let [less, equal, greater] = facts.members.variants(owner) else {
            return Err("partial_cmp ordering has incorrect variants");
        };
        let (less, equal, greater) = (*less, *equal, *greater);
        let field = facts
            .members
            .field(some, crate::sema::members::Name::Index(0))
            .ok_or("partial_cmp option has no payload field")?;
        Ok(Self {
            ordering,
            some,
            less,
            equal,
            greater,
            field,
        })
    }
}
fn initialize(
    definition: DefId,
    signature: TypeId,
    source: NodeKey,
    parameters: [TypeId; 2],
    result: TypeId,
) -> Result<Body, &'static str> {
    let boolean = TypeId::primitive(Primitive::Bool);
    let mut body = Body::new(
        Owner::Function {
            definition,
            signature,
            execution: None,
        },
        source,
        boolean,
    );
    body.storage = Some(crate::body::storage::Facts::new(scopes::Facts::synthetic(
        source.node,
    )));
    for (index, ty) in parameters
        .into_iter()
        .chain(std::iter::once(result))
        .enumerate()
    {
        let index = u32::try_from(index).map_err(|_| "too many provided parameters")?;
        body.locals.push(Local {
            scope: scopes::Id(0),
            storage_parent: index.checked_sub(1).map(LocalId),
            ty,
            kind: if index < 2 {
                LocalKind::Parameter(index)
            } else {
                LocalKind::Binding
            },
            definition: None,
            mutable: false,
            source: source.node.into(),
        });
        statement(&mut body, Action::StorageLive(LocalId(index)));
        if index < 2 {
            let read = value(&mut body, ty, Rvalue::Read(Place::Local(LocalId(index))))?;
            body.arguments.push(read);
        }
    }
    Ok(body)
}
