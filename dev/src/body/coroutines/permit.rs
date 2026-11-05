//! Compiler construction origin is distinct from the one checked deferred body.
use crate::{
    body::ir::{Body, Owner},
    resolve::unit::NodeKey,
    types::{Kind, Store, TypeId, arena::Identity as Arena, instance::Instance, opaque::OpaqueId},
};

pub(crate) struct Permit {
    pub(super) identity: Identity,
    pub(super) future: TypeId,
    pub(super) public: TypeId,
    pub(super) execution: TypeId,
    pub(super) source: NodeKey,
    pub(super) arena: Arena,
}
pub(super) enum Identity {
    Constructor(Instance),
    Drop,
}
impl Permit {
    pub(crate) const fn instance(&self) -> Option<Instance> {
        match self.identity {
            Identity::Constructor(instance) => Some(instance),
            Identity::Drop => None,
        }
    }
    pub(crate) const fn entry(&self) -> Option<crate::types::future::EntryKey> {
        match self.identity {
            Identity::Constructor(_) => None,
            Identity::Drop => Some(crate::types::future::EntryKey {
                future: self.future,
                operation: crate::types::future::Operation::Drop,
            }),
        }
    }
    /// The validated immutable family lends only its actual compiler state read.
    pub(crate) fn state_read(
        &self,
        types: &Store,
        body: &Body,
        place: crate::body::ir::Place,
    ) -> Result<bool, &'static str> {
        use crate::body::ir::{Place, ProjectionId, ProjectionKind};
        if self.entry().is_none() || place != Place::Projection(ProjectionId(1)) {
            return Ok(false);
        }
        self.family(types)?;
        if body.source != self.source.into()
            || !matches!(body.owner, Owner::FutureEntry { key, signature, execution } if Some(key)==self.entry() && signature==self.public && execution==self.execution)
        {
            return Err("compiler state read changed its actual generated family");
        }
        let Some(state) = body.projections.get(1) else {
            return Err("compiler state read lost its actual place");
        };
        if state.base != Place::Projection(ProjectionId(0))
            || state.ty != TypeId::primitive(crate::types::Primitive::U8)
            || !matches!(state.kind, ProjectionKind::FutureField(0))
        {
            return Err("compiler state read differs from its actual typed state coordinate");
        }
        Ok(true)
    }
    pub(crate) const fn future(&self) -> TypeId {
        self.future
    }
    pub(crate) const fn source(&self) -> NodeKey {
        self.source
    }
    pub(crate) fn family(&self, types: &Store) -> Result<OpaqueId, &'static str> {
        if !types.has_identity(&self.arena) {
            return Err("future constructor permit belongs to another actual arena");
        }
        let Some(Kind::Future { identity, .. }) = types.kind(self.future) else {
            return Err("future constructor permit lost its actual complete future");
        };
        Ok(identity)
    }
    pub(crate) fn require_current(&self, types: &Store, body: &Body) -> Result<(), &'static str> {
        self.family(types)?;
        if body.static_types.is_some()
            || body.mutable_statics.is_some()
            || body.promotions.is_some()
            || body.closure_calls.is_some()
            || body.has_resolved_callables()
        {
            return Err("generated future entry has unrelated lexical or callable payloads");
        }
        let Some(instance) = self.instance() else {
            if body.instance.is_some()
                || body.source != self.source.into()
                || body.result != TypeId::UNIT
                || body.const_context
                || !matches!(body.owner, Owner::FutureEntry { key, signature, execution } if Some(key)==self.entry() && signature==self.public && execution==self.execution)
            {
                return Err("future Drop changed its actual family, origin or signature worlds");
            }
            return super::disposal::validate(types, body, self);
        };
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = types.kind(instance.ty())
        else {
            return Err("future constructor permit lost its real public function item");
        };
        if signature != self.public
            || body.instance != Some(instance)
            || body.source != self.source.into()
            || body.result != self.future
            || body.const_context
            || !matches!(body.owner, Owner::Function {definition, signature, execution:Some(execution)}
                if definition.0 == item.0 && signature == self.public && execution == self.execution)
        {
            return Err(
                "future constructor changed its checked origin, full instance or signature worlds",
            );
        }
        construction(types, body, self.future, self.source)
    }
}

fn construction(
    types: &Store,
    body: &Body,
    future: TypeId,
    source: NodeKey,
) -> Result<(), &'static str> {
    use crate::body::{
        ir::{Action, BlockId, Rvalue, ScalarId, Value, ValueId, ValueList},
        scopes::{Id, Lifetime},
        storage::{End, Location, Range},
    };
    let Some(Kind::Future {
        identity,
        arguments,
    }) = types.kind(future)
    else {
        return Err("future construction lost its actual identity");
    };
    let storage = types.future_application(identity, arguments)?.storage;
    let byte = TypeId::primitive(crate::types::Primitive::U8);
    let facts = body
        .storage
        .as_ref()
        .ok_or("future construction lost its actual storage scope")?;
    let valid_scopes = matches!(facts.scopes.as_slice(), [scope] if scope.parent.is_none() && scope.kind==crate::body::scopes::Kind::Body && scope.source==source.node.into());
    let valid_exit = matches!(facts.exits.as_slice(), [exit] if exit.location==Location::At {block:BlockId(0),before:2} && exit.end==End::Scopes(Range {inner:Id(0),outer:None}));
    let valid_values = matches!(body.values.as_slice(), [Value {ty:state,kind:Rvalue::Scalar(ScalarId(0))},Value {ty:actual,kind:Rvalue::Aggregate {variant:None,fields:ValueList {start:0,end:1}}}] if *state==byte && *actual==future);
    let valid_actions = matches!(body.statements.as_slice(), [first,second] if first.source==source.node.into() && second.source==source.node.into()
        && matches!(first.action,Action::Evaluate(ValueId(0))) && matches!(second.action,Action::Evaluate(ValueId(1))));
    let owned = body
        .owned
        .as_deref()
        .ok_or("future construction lost its actual owned return")?;
    let owned_valid = matches!(owned.created.as_slice(), [value] if value.value==ValueId(1) && value.lifetime==Lifetime::Scope(Id(0)) && value.block==BlockId(0) && value.before==2 && value.exits_before==0)
        && matches!(owned.returned.as_slice(), [value] if value.value==ValueId(1) && value.location==Location::At {block:BlockId(0),before:2} && value.exits_before==0);
    if types.list(storage) != Some([byte].as_slice())
        || body.assembly.is_some()
        || !body.locals.is_empty()
        || !body.projections.is_empty()
        || !body.temporaries.is_empty()
        || !body.guards.is_empty()
        || !body.guard_places.is_empty()
        || body.values.len() != 2
        || body.scalars != [crate::sema::model::Scalar::Integer(0)]
        || body.arguments != [ValueId(0)]
        || !valid_values
        || !valid_actions
        || !valid_scopes
        || !valid_exit
        || !owned_valid
        || !matches!(body.blocks.as_slice(), [block] if block.start==0 && block.end==2 && matches!(block.terminator,crate::body::ir::Terminator::Return(ValueId(1))))
    {
        return Err("future construction differs from its exact dormant storage/return contract");
    }
    Ok(())
}
const _: [(); 40] = [(); size_of::<Permit>()];
