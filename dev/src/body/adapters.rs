//! Receiver wrappers retain distinct identities and ordinary local borrow/cleanup duties.
use super::{
    invocation_entries::Entry,
    ir::{
        Action, Body, BodyKey, BorrowKind, Local, LocalId, LocalKind, Owner, Place, Projection,
        ProjectionId, ProjectionKind, Rvalue, Statement, Value, ValueId, ValueList,
    },
    regions::{Contract, Kind as ObligationKind, Obligation},
};
use crate::{
    sema::model::Declarations,
    support::call_traits::CallTrait,
    types::{Kind, RegionId, TypeId, closure::CallKind},
};

/// A derived callable interface still requires its sealed original source proof.
pub(crate) struct Template {
    adapter: crate::types::closure::Adapter,
    source: crate::resolve::unit::NodeKey,
    public: TypeId,
    original: TypeId,
    signature: TypeId,
}
pub(crate) struct Prepared {
    pub body: Body,
    pub input: Contract,
    pub template: Template,
}
impl Template {
    pub(crate) fn validate(
        &self,
        types: &crate::types::Store,
        original: &Body,
        body: &Body,
    ) -> Result<(), &'static str> {
        let Some(Kind::Closure { identity, .. }) = types.kind(self.adapter.instance().ty()) else {
            return Err("adapter template has no canonical closure instance");
        };
        if crate::types::closure::Adapter::new(
            types,
            self.adapter.instance(),
            self.adapter.requested(),
        )? != self.adapter
            || original.instance.is_some()
            || original.source != self.source
            || !matches!(original.owner, Owner::Closure { identity: actual, signature, execution: Some(execution) }
                if identity == actual && signature == self.public && execution == self.original)
        {
            return Err("adapter template belongs to a different proved source");
        }
        super::invocation_entries::validate_instance(
            types,
            self.original,
            self.adapter.instance(),
        )?;
        if body.source != self.source
            || body.instance != Some(self.adapter.instance())
            || !matches!(body.owner, Owner::ClosureAdapter { adapter, signature }
                if adapter == self.adapter && signature == self.signature)
        {
            return Err("adapter template belongs to a different materialized wrapper");
        }
        Ok(())
    }
}

pub(crate) fn build(
    facts: &mut Declarations,
    selected: Entry,
    original: &Body,
) -> Result<Prepared, &'static str> {
    let adapter = selected
        .adapter()
        .ok_or("missing selected receiver adapter")?;
    let (source, execution) = selected.validate_adapter(facts)?;
    let canonical = super::invocation_entries::validate_execution(&facts.types, source)?;
    if original.instance.is_some()
        || original.source != source.semantic_source()
        || !matches!(original.owner, Owner::Closure { identity, signature, execution: Some(body) }
            if identity == source.identity && signature == source.signature
                && body == canonical.signature)
    {
        return Err("receiver adapter selects a different original source body");
    }
    let Some(Kind::Function {
        parameters,
        result,
        convention,
        ..
    }) = facts.types.kind(execution.signature)
    else {
        return Err("receiver adapter has no callable execution");
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("missing adapter parameters")?
        .to_vec();
    let mut writer = Writer::new(source, adapter, execution, &parameters, result)?;
    let mut contract = contract(source.semantic_source(), adapter, execution);
    let result = writer.invoke(
        &mut facts.types,
        canonical,
        execution,
        &parameters,
        result,
        convention,
    )?;
    super::build::finish_factory(&mut writer.body, result)?;
    super::invocation_entries::select(facts, &mut writer.body, &mut contract)?;
    let template = Template {
        adapter,
        source: source.semantic_source(),
        public: source.signature,
        original: canonical.signature,
        signature: execution.signature,
    };
    template.validate(&facts.types, original, &writer.body)?;
    Ok(Prepared {
        body: writer.body,
        input: contract,
        template,
    })
}

fn contract(
    source: crate::resolve::unit::NodeKey,
    adapter: crate::types::closure::Adapter,
    execution: crate::sema::closures::Execution,
) -> Contract {
    // The private borrow never replaces a source universal or receives assumptions.
    let region = RegionId(1);
    Contract {
        key: BodyKey::ClosureAdapter(adapter),
        source,
        signature: Some(execution.signature),
        universals: execution.universals,
        assumptions: Vec::new(),
        parameters: Vec::new(),
        type_assumptions: Vec::new(),
        obligations: vec![
            Obligation {
                source,
                kind: ObligationKind::Variable {
                    id: region,
                    needs_provenance: true,
                },
            },
            Obligation {
                source,
                kind: ObligationKind::LocalBorrow(region),
            },
        ],
        proofs: Vec::new(),
        builtins: None,
        output: None,
    }
}

struct Writer {
    body: Body,
}
impl Writer {
    fn new(
        source: crate::sema::closures::Body,
        adapter: crate::types::closure::Adapter,
        execution: crate::sema::closures::Execution,
        parameters: &[TypeId],
        result: TypeId,
    ) -> Result<Self, &'static str> {
        let mut body = Body::new(
            Owner::ClosureAdapter {
                adapter,
                signature: execution.signature,
            },
            source.semantic_source(),
            result,
        );
        body.instance = Some(adapter.instance());
        body.storage = Some(super::storage::Facts::new(super::scopes::Facts::synthetic(
            body.source.node,
        )));
        let mut writer = Self { body };
        for (index, &ty) in parameters.iter().enumerate() {
            let index = u32::try_from(index).map_err(|_| "too many adapter parameters")?;
            writer.body.locals.push(Local {
                scope: super::scopes::Id(0),
                storage_parent: index.checked_sub(1).map(LocalId),
                ty,
                kind: LocalKind::Parameter(index),
                definition: None,
                mutable: index == 0 && execution.receiver == CallKind::Once,
                source: source.semantic_source().node,
            });
            writer.statement(Action::StorageLive(LocalId(index)));
        }
        Ok(writer)
    }
    fn invoke(
        &mut self,
        types: &mut crate::types::Store,
        canonical: crate::sema::closures::Execution,
        execution: crate::sema::closures::Execution,
        parameters: &[TypeId],
        result: TypeId,
        convention: crate::types::callable::Convention,
    ) -> Result<ValueId, &'static str> {
        let region = RegionId(1);
        let place = self.receiver(types, parameters[0], execution.receiver)?;
        let receiver = types.intern(Kind::Reference {
            referent: canonical.self_ty,
            mutable: canonical.receiver == CallKind::Mutable,
            region,
        })?;
        let receiver = self.value(
            receiver,
            Rvalue::Borrow {
                place,
                kind: BorrowKind::Immediate,
            },
        )?;
        let mut inputs = vec![receiver];
        for (index, &ty) in parameters.iter().enumerate().skip(1) {
            inputs.push(self.value(
                ty,
                Rvalue::Read(Place::Local(LocalId(
                    u32::try_from(index).map_err(|_| "too many adapter arguments")?,
                ))),
            )?);
        }
        self.body.arguments.extend(inputs);
        let arguments = ValueList {
            start: 0,
            end: u32::try_from(parameters.len()).map_err(|_| "too many adapter inputs")?,
        };
        let parameters = types.intern_list(&parameters[1..])?;
        let signature = types.intern(Kind::Function {
            parameters,
            result,
            bound_regions: 0,
            unsafe_: false,
            convention,
        })?;
        self.value(
            result,
            Rvalue::Invoke {
                kind: match canonical.receiver {
                    CallKind::Shared => CallTrait::Fn,
                    CallKind::Mutable => CallTrait::FnMut,
                    CallKind::Once => {
                        return Err("receiver adapter cannot borrow a once-only source");
                    }
                },
                signature,
                arguments,
            },
        )
    }

    fn statement(&mut self, action: Action) {
        self.body.statements.push(Statement {
            source: self.body.source.node,
            action,
        });
    }
    fn value(&mut self, ty: TypeId, kind: Rvalue) -> Result<ValueId, &'static str> {
        let id =
            ValueId(u32::try_from(self.body.values.len()).map_err(|_| "too many adapter values")?);
        self.body.values.push(Value { ty, kind });
        self.statement(Action::Evaluate(id));
        Ok(id)
    }
    fn receiver(
        &mut self,
        types: &crate::types::Store,
        ty: TypeId,
        permission: CallKind,
    ) -> Result<Place, &'static str> {
        if permission == CallKind::Once {
            return Ok(Place::Local(LocalId(0)));
        }
        let Some(Kind::Reference {
            referent,
            region,
            mutable,
        }) = types.kind(ty)
        else {
            return Err("borrowed adapter receiver is not a reference");
        };
        let id = ProjectionId(
            u32::try_from(self.body.projections.len())
                .map_err(|_| "too many adapter projections")?,
        );
        self.body.projections.push(Projection {
            base: Place::Local(LocalId(0)),
            kind: ProjectionKind::Dereference { region, mutable },
            ty: referent,
        });
        self.statement(Action::Project(id));
        Ok(Place::Projection(id))
    }
}
