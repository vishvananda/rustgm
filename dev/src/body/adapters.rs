//! Receiver wrappers retain distinct identities and ordinary local borrow/cleanup duties.
mod preconditions;
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
    instance: Option<crate::types::instance::Instance>,
    receiver: TypeId,
    signature: TypeId,
    requirements: preconditions::Requirements,
}
pub(crate) struct Prepared {
    pub body: Body,
    pub input: Contract,
    pub template: Template,
}
impl Template {
    pub(crate) fn validate_contract(
        &self,
        types: &crate::types::Store,
        input: &Contract,
    ) -> Result<(), &'static str> {
        let Some(Kind::Function { bound_regions, .. }) = types.kind(self.signature) else {
            return Err("receiver adapter has no declared execution signature");
        };
        if input.key != BodyKey::ClosureAdapter(self.adapter)
            || input.source != self.source.into()
            || input.signature != Some(self.signature)
            || input.universals != bound_regions
        {
            return Err("receiver adapter has a different declared interface");
        }
        self.requirements.validate(input)
    }
    pub(crate) fn validate(
        &self,
        types: &crate::types::Store,
        original: &Body,
        body: &Body,
    ) -> Result<(), &'static str> {
        let Some(Kind::Closure { identity, .. }) = types.kind(self.adapter.instance().ty()) else {
            return Err("adapter template has no canonical closure instance");
        };
        self.adapter.validate(types)?;
        if original.instance != self.instance
            || original.source != self.source.into()
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
        super::invocation_entries::validate_receiver(
            types,
            self.signature,
            self.receiver,
            self.adapter,
        )?;
        if body.source != self.source.into()
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
    let (source, prepared) = selected.validate_adapter(facts)?;
    let execution = prepared.execution;
    let canonical = prepared.canonical;
    let generic = source
        .owner
        .is_some_and(|owner| facts.generics.has_parameters(owner));
    let instance = generic.then_some(adapter.instance());
    let public = if generic {
        let Some(Kind::Closure { signature, .. }) = facts.types.kind(adapter.instance().ty())
        else {
            return Err("adapter source has no closure signature");
        };
        signature
    } else {
        source.signature
    };
    let Owner::Closure {
        identity,
        signature,
        execution: Some(original_execution),
    } = original.owner
    else {
        return Err("receiver adapter requires a proved closure source");
    };
    if original.instance != instance
        || original.source != source.semantic_source().into()
        || identity != source.identity
        || signature != public
        || (!generic && original_execution != canonical.signature)
    {
        return Err("receiver adapter selects a different original source body");
    }
    super::invocation_entries::validate_instance(
        &facts.types,
        original_execution,
        adapter.instance(),
    )?;
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
    let requirements = preconditions::selected(facts, source, canonical, adapter)?;
    contract.assumptions.extend_from_slice(requirements.pairs());
    for proof in prepared.proofs {
        crate::body::instantiate::retain_closure_proof(
            facts,
            &mut contract,
            proof,
            (0..execution.universals)
                .map(|slot| {
                    let slot = u16::try_from(slot).map_err(|_| "too many adapter lifetimes")?;
                    Ok((slot, RegionId::bound(0, slot)?))
                })
                .collect::<Result<Vec<_>, &'static str>>()?
                .into_iter(),
        )?;
    }
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
    super::copy_requirements::prepare(facts, &writer.body, &mut contract)
        .map_err(|_| "receiver adapter source Copy preparation failed")?;
    let template = Template {
        adapter,
        source: source.semantic_source(),
        public,
        original: original_execution,
        instance,
        receiver: execution.self_ty,
        signature: execution.signature,
        requirements,
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
    // The private borrow never replaces a source universal or acquires a premise.
    let region = RegionId(1);
    Contract {
        key: BodyKey::ClosureAdapter(adapter),
        source: source.into(),
        signature: Some(execution.signature),
        universals: execution.universals,
        assumptions: Vec::new(),
        parameters: Vec::new(),
        type_assumptions: Vec::new(),
        obligations: vec![
            Obligation {
                source: source.into(),
                kind: ObligationKind::Variable {
                    id: region,
                    needs_provenance: true,
                },
            },
            Obligation {
                source: source.into(),
                kind: ObligationKind::LocalBorrow(region),
            },
        ],
        proofs: Vec::new(),
        builtins: None,
        output: None,
        require_result_formation: false,
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
            source.semantic_source().node,
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
                source: source.semantic_source().node.into(),
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
        let place = self.referent(types, place, execution.self_ty, canonical.self_ty)?;
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
            source: self.body.root_site(),
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
    fn referent(
        &mut self,
        types: &crate::types::Store,
        mut place: Place,
        mut self_ty: TypeId,
        closure: TypeId,
    ) -> Result<Place, &'static str> {
        let mut depth = 0usize;
        while self_ty != closure {
            depth += 1;
            if depth > 1_000_000 {
                return Err("adapter reference path exceeds its work budget");
            }
            let Some(Kind::Reference {
                referent,
                mutable,
                region,
            }) = types.kind(self_ty)
            else {
                return Err("borrowed receiver Self is not a reference to its closure");
            };
            let id = ProjectionId(
                u32::try_from(self.body.projections.len())
                    .map_err(|_| "too many adapter projections")?,
            );
            self.body.projections.push(Projection {
                base: place,
                kind: ProjectionKind::Dereference { mutable, region },
                ty: referent,
            });
            self.statement(Action::Project(id));
            place = Place::Projection(id);
            self_ty = referent;
        }
        Ok(place)
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
