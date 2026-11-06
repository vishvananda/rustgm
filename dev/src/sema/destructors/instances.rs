//! Select destructor code from checked nominal coverage, without repeating source matching.
use super::Implementation;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        GenericArgs, ItemId, Kind, RegionId, TypeId, arguments::Argument, instance::Instance,
        substitute::Bound,
    },
};

/// Select the complete callable instance for an already checked concrete nominal type.
/// Unused impl lifetime slots stay in the original proof; they do not acquire
/// arbitrary lifetime arguments merely to form a runtime callable signature.
/// # Errors
/// Rejects symbolic targets, missing coverage/signatures and invalid substitutions.
pub fn select(facts: &mut Declarations, target: TypeId) -> Result<Option<Instance>, &'static str> {
    if let Some(&instance) = facts.destructors.instances.get(&target) {
        return Ok(Some(instance));
    }
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(target) else {
        return Ok(None);
    };
    let Some(implementation) = facts.destructors.item(item) else {
        return Ok(None);
    };
    if facts.types.contains_parameters(target) != Some(false) {
        return Err("destructor selection requires concrete type and const arguments");
    }
    let method = implementation.method;
    let owner = implementation.self_definition;
    let inverse = implementation.coverage.arguments;
    if let Some(instance) = provided_instance(facts, target, method, owner)? {
        return Ok(Some(instance));
    }
    let definition = facts
        .definitions
        .get(method.0 as usize)
        .copied()
        .flatten()
        .ok_or("missing destructor function item")?;
    // Lifetime-only methods retain the existing universally checked concrete body.
    if !facts.generics.has_parameters(method) {
        return Instance::new(&facts.types, definition).map(Some);
    }
    if facts.lifetimes.callables.get(ItemId(method.0)).is_none() {
        publish(facts, method)?;
    }
    let mut substitution = Bound::joint(&facts.types, item, arguments)?;
    let mut actual = facts
        .types
        .arguments(arguments)
        .ok_or("invalid destructor target arguments")?
        .filter(|a| matches!(a, Argument::Lifetime(_)))
        .collect::<Vec<_>>();
    let formals = facts
        .types
        .arguments(inverse)
        .ok_or("missing destructor coverage arguments")?
        .filter(|a| !matches!(a, Argument::Lifetime(_)))
        .collect::<Vec<_>>();
    for formal in formals {
        actual.push(substitution.apply_argument(&mut facts.types, formal)?);
    }
    let arguments = facts.types.intern_arguments(&actual)?;
    let parameters = facts.generics.parameter_ids(&facts.types, owner)?;
    let early = actual
        .iter()
        .take_while(|a| matches!(a, Argument::Lifetime(_)))
        .count();
    if parameters.len() != actual.len() - early {
        return Err("destructor coverage has different parameter arity");
    }
    let bindings = parameters
        .into_iter()
        .zip(actual[early..].iter().copied())
        .collect::<Vec<_>>();
    let regions = actual[..early]
        .iter()
        .filter_map(|a| {
            if let Argument::Lifetime(region) = a {
                Some(*region)
            } else {
                None
            }
        })
        .collect();
    let template = facts
        .lifetimes
        .callables
        .get(ItemId(method.0))
        .ok_or("missing destructor callable")?;
    let signature = Bound::parameters_captured(&facts.types, &bindings, regions)?
        .apply(&mut facts.types, template.signature)?;
    let selected = facts.types.intern(Kind::FunctionItem {
        item: ItemId(method.0),
        signature,
        arguments,
    })?;
    let instance = Instance::new(&facts.types, selected)?;
    crate::types::destruction::validate_method(
        &mut facts.types,
        instance,
        ItemId(method.0),
        target,
    )?;
    facts.destructors.instances.insert(target, instance);
    Ok(Some(instance))
}

/// Select the actual provided `Drop::drop` body for a pin-only implementation.
/// This is callable data; its original body still passes normal materialization.
fn provided_instance(
    facts: &mut Declarations,
    target: TypeId,
    method: DefId,
    owner: DefId,
) -> Result<Option<Instance>, &'static str> {
    let declaration = cleanup_method(facts, method, owner)?;
    if declaration == method {
        return Ok(None);
    }
    let function = facts
        .function_indices
        .get(&declaration)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("provided destructor lost its actual source function")?;
    if function.definition != declaration || function.body.is_none() {
        return Err("pinned destruction requires its actual provided Drop body");
    }
    let item = ItemId(declaration.0);
    let template = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("provided Drop lacks its canonical callable")?;
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(item)
        .ok_or("provided Drop lacks its actual parameter table")?;
    let [parameter] = parameters else {
        return Err("provided Drop has another generic contract");
    };
    if template.parameters != 0 {
        return Err("provided Drop has unsupported early lifetimes");
    }
    let signature = template.signature;
    let bindings = [(*parameter, Argument::Type(target))];
    let payments =
        crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
            .map_err(|_| "provided Drop requires its actual source payer")?;
    let pay = |work, bytes| payments.charge(work, bytes);
    let mut fold = Bound::parameters_captured_charged(&facts.types, &bindings, Vec::new(), &pay)?;
    let Argument::Type(signature) =
        fold.apply_argument_charged(&mut facts.types, Argument::Type(signature), &pay)?
    else {
        return Err("provided Drop substitution changed its signature kind");
    };
    let arguments = facts
        .types
        .intern_arguments_observed::<true>(&[Argument::Type(target)], &pay)?;
    let ty = facts.types.intern_observed::<true>(
        Kind::FunctionItem {
            item,
            signature,
            arguments,
        },
        &pay,
    )?;
    let instance = Instance::new(&facts.types, ty)?;
    crate::types::destruction::validate_method(&mut facts.types, instance, item, target)?;
    let growth = crate::support::retained::Growth::map::<TypeId, Instance>(
        facts.destructors.instances.len(),
        facts.destructors.instances.capacity(),
        1,
    );
    pay(growth.work, growth.bytes)?;
    facts.destructors.instances.insert(target, instance);
    Ok(Some(instance))
}

/// The recipe names its actual callable, independently of the impl's override.
pub(crate) fn cleanup_method(
    facts: &Declarations,
    method: DefId,
    owner: DefId,
) -> Result<DefId, &'static str> {
    let signature = facts
        .function_header(method)?
        .ok_or("destructor method header is pending")?
        .signature;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("destructor method lost its actual signature");
    };
    let Some([receiver]) = facts.types.list(parameters) else {
        return Err("destructor method lost its actual receiver");
    };
    if matches!(facts.types.kind(*receiver), Some(Kind::Reference { .. })) {
        return Ok(method);
    }
    provided_member(facts, owner, method)
}

fn provided_member(
    facts: &Declarations,
    owner: DefId,
    actual: DefId,
) -> Result<DefId, &'static str> {
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("pinned destruction requires source Drop")?
        .validated_language(crate::support::bindings::LangItem::Drop)?
        .ok_or("pinned destruction lost its actual Drop role")?;
    let registry = facts
        .generics
        .implementations
        .as_deref()
        .ok_or("missing destructor implementation registry")?;
    if !registry.is_validated(owner) {
        return Err("pinned destruction requires its checked implementation");
    }
    let header = registry
        .header(owner)
        .ok_or("pinned destruction lost its actual implementation")?;
    let mut found = None;
    let mut implemented = false;
    for &(member, selected) in &header.methods {
        if facts.associated.trait_owner(member) != Some(role.definition) {
            return Err("destructor method mapping changed its actual trait owner");
        }
        implemented |= selected == Some(actual);
        let signature = facts
            .function_header(member)?
            .ok_or("Drop member signature is pending")?
            .signature;
        let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
            return Err("Drop member is not a callable");
        };
        let Some([receiver]) = facts.types.list(parameters) else {
            return Err("Drop member has another receiver contract");
        };
        if matches!(facts.types.kind(*receiver), Some(Kind::Reference { .. }))
            && (selected.is_some() || found.replace(member).is_some())
        {
            return Err(
                "pinned destruction cannot replace or conflict with its provided drop member",
            );
        }
    }
    if !implemented {
        return Err("pinned destruction lost its actual override mapping");
    }
    found.ok_or("pinned destruction has no actual provided Drop member")
}

fn publish(facts: &mut Declarations, method: DefId) -> Result<(), &'static str> {
    let implementation = facts
        .destructors
        .method(method)
        .ok_or("missing destructor implementation")?;
    let owner = implementation.self_definition;
    let scheme = facts
        .lifetimes
        .functions
        .get(&method)
        .ok_or("missing destructor lifetime scheme")?;
    let definition = facts
        .definitions
        .get(method.0 as usize)
        .copied()
        .flatten()
        .ok_or("missing destructor definition")?;
    // The definition retains the original body signature, including every
    // universal. Only this separately published callable is compacted.
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(definition)
    else {
        return Err("missing destructor function signature");
    };
    if item.0 != method.0 {
        return Err("destructor definition has a foreign callable identity");
    }
    let (mapping, early) = mapping(facts, implementation, scheme)?;
    validate_unused(&facts.types, signature, &mapping)?;
    let signature = Bound::new(mapping).signature(&mut facts.types, signature, 1)?;
    let parameters = facts.generics.parameter_ids(&facts.types, owner)?;
    facts.lifetimes.callables.insert_inherited(
        &facts.types,
        ItemId(method.0),
        ItemId(owner.0),
        signature,
        early,
        &parameters,
    )
}

fn validate_unused(
    types: &crate::types::Store,
    signature: TypeId,
    mapping: &[RegionId],
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        parameters, result, ..
    }) = types.kind(signature)
    else {
        return Err("missing destructor method signature");
    };
    let mut uses = crate::types::bound_uses::Uses::default();
    for &root in types
        .list(parameters)
        .ok_or("invalid destructor parameters")?
    {
        uses.collect(types, root)?;
    }
    uses.collect(types, result)?;
    if uses.indices.iter().any(|&index| {
        mapping
            .get(usize::from(index))
            .is_none_or(|&region| region == RegionId::STATIC)
    }) {
        return Err("destructor callable cannot erase a used impl lifetime");
    }
    Ok(())
}

fn mapping(
    facts: &Declarations,
    implementation: &Implementation,
    scheme: &crate::sema::lifetimes::FunctionRegions,
) -> Result<(Vec<RegionId>, u32), &'static str> {
    let nominal = facts
        .definitions
        .get(implementation.item.0 as usize)
        .copied()
        .flatten()
        .ok_or("missing destructor nominal declaration")?;
    let Some(Kind::Adt(_, arguments)) = facts.types.kind(nominal) else {
        return Err("invalid destructor nominal declaration");
    };
    let early = lifetime_count(&facts.types, arguments)?;
    let mut mapping = vec![RegionId::STATIC; scheme.total as usize];
    for (slot, &region) in implementation.coverage.regions.iter().enumerate() {
        let Some((0, index)) = region.bound_parts() else {
            return Err("invalid destructor capture mapping");
        };
        if u32::from(index) < early {
            *mapping
                .get_mut(slot)
                .ok_or("missing destructor capture slot")? = RegionId::bound(1, index)?;
        }
    }
    let [receiver] = scheme.late.as_slice() else {
        return Err("destructor must have one late receiver lifetime");
    };
    *mapping
        .get_mut(usize::from(*receiver))
        .ok_or("missing destructor receiver slot")? = RegionId::bound(0, 0)?;
    Ok((mapping, early))
}
fn lifetime_count(
    types: &crate::types::Store,
    arguments: GenericArgs,
) -> Result<u32, &'static str> {
    let count = types
        .arguments(arguments)
        .ok_or("invalid nominal argument schema")?
        .take_while(|a| matches!(a, Argument::Lifetime(_)))
        .count();
    u32::try_from(count).map_err(|_| "too many destructor target lifetimes")
}
