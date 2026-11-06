//! Poll ABI data borrows real source declarations independently of executable state.
use crate::{
    resolve::{bindings::Declaration, model::DefId},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{
        GenericArgs, ItemId, Kind, RegionId, TypeId, aggregate, arguments::Argument,
        callable::Convention,
    },
};

pub(crate) struct Declared {
    pub method: DefId,
    pub signature: TypeId,
    pub self_type: TypeId,
    pub poll: ItemId,
    pub pin_field: crate::sema::members::FieldId,
    pub ready: crate::sema::members::VariantId,
}

pub(super) fn require(facts: &mut Declarations) -> Result<Declared, &'static str> {
    let (future, output) = super::schema::require(facts)?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("poll protocol lost its actual catalog")?;
    let method = catalog
        .validated_language(LangItem::FuturePoll)?
        .ok_or("poll operation requires its actual completed member")?;
    let header = facts
        .function_header(method.definition)?
        .ok_or("poll operation lost its actual callable header")?;
    let self_type = facts
        .associated
        .schema(future)
        .ok_or("poll operation lost its actual Self schema")?
        .self_type;
    let scheme = facts
        .lifetimes
        .functions
        .get(&method.definition)
        .ok_or("poll operation lost its actual public binder frame")?;
    facts.charge_caller_inputs(20, 0)?;
    if method.parent != Some(future)
        || facts.associated.trait_owner(method.definition) != Some(future)
        || header.declaration.source() != Some(method.origin)
        || header.body.is_some()
        || facts.is_const_function(method.definition)
        || scheme.captures != 0
        || !scheme.early.is_empty()
        || !scheme.bounds.is_empty()
        || scheme.late.len() != 3
        || facts
            .generics
            .parameters
            .get(&method.definition)
            .is_some_and(|parameters| !parameters.is_empty())
        || crate::sema::declaration_inputs::own_view(
            &facts.generics,
            facts.crates.as_deref(),
            method.definition,
        )?
        .len()
            != 0
    {
        return Err(
            "demanded poll requires its actual ordinary required member and three independent public binder slots",
        );
    }
    let source_signature = header.signature;
    let public = public_signature(facts, method.definition, source_signature)?;
    let (pin, pin_arguments) = nominal(facts, LangItem::Pin, aggregate::Kind::Struct)?;
    let (poll, poll_arguments) = nominal(facts, LangItem::Poll, aggregate::Kind::Enum)?;
    let (context, context_arguments) = nominal(facts, LangItem::Context, aggregate::Kind::Struct)?;
    let pin_parameter = type_parameter(facts, pin, pin_arguments)?;
    let poll_parameter = type_parameter(facts, poll, poll_arguments)?;
    let mut context_parameters = facts
        .types
        .arguments(context_arguments)
        .ok_or("poll Context lost its actual lifetime schema")?;
    if !matches!(context_parameters.next(),Some(Argument::Lifetime(region)) if region == RegionId::bound(0,0)?)
        || context_parameters.next().is_some()
    {
        return Err("demanded poll Context requires its actual one-lifetime nominal schema");
    }
    let pin_field = pin_field(facts, pin, pin_parameter)?;
    let ready = variants(facts, poll, poll_parameter)?;
    signature(facts, public, self_type, pin, poll, context, output)?;
    Ok(Declared {
        method: method.definition,
        signature: public,
        self_type,
        poll,
        pin_field,
        ready,
    })
}

fn nominal(
    facts: &Declarations,
    role: LangItem,
    kind: aggregate::Kind,
) -> Result<(ItemId, GenericArgs), &'static str> {
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("poll nominal lost its actual catalog")?
        .validated_language(role)?
        .ok_or("poll operation requires its actual completed nominal role")?;
    let item = ItemId(declaration.definition.0);
    let Some(Kind::Adt(actual, arguments)) = facts
        .definitions
        .get(item.0 as usize)
        .copied()
        .flatten()
        .and_then(|ty| facts.types.kind(ty))
    else {
        return Err("poll nominal lost its actual declared type");
    };
    if actual != item
        || !facts
            .generics
            .nominal_contracts
            .contains(&declaration.definition)
        || facts
            .aggregates
            .get(item)
            .is_none_or(|definition| definition.kind != kind)
    {
        return Err("poll nominal differs from its actual completed source/schema");
    }
    Ok((item, arguments))
}

fn type_parameter(
    facts: &Declarations,
    owner: ItemId,
    arguments: GenericArgs,
) -> Result<TypeId, &'static str> {
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("poll nominal lost its formal arguments")?;
    let Some(Argument::Type(ty)) = arguments.next() else {
        return Err("poll nominal requires its actual single type formal");
    };
    if arguments.next().is_some()
        || !matches!(facts.types.kind(ty),Some(Kind::Parameter(id))
        if facts.types.parameter(id).is_some_and(|formal| formal.owner == owner && formal.index == 0))
    {
        return Err("poll nominal requires its actual single type formal");
    }
    Ok(ty)
}

fn pin_field(
    facts: &Declarations,
    pin: ItemId,
    parameter: TypeId,
) -> Result<crate::sema::members::FieldId, &'static str> {
    let [variant] = facts.members.variants(pin) else {
        return Err("demanded poll Pin requires its actual single storage variant");
    };
    let fields = facts
        .members
        .fields(*variant)
        .ok_or("poll Pin fields are not complete")?;
    if !matches!(fields,[field] if field.ty == parameter) {
        return Err("demanded poll Pin requires its actual pointer parameter field");
    }
    let member = facts
        .members
        .variants
        .get(variant.0 as usize)
        .ok_or("poll Pin lost its actual storage member")?;
    Ok(crate::sema::members::FieldId(member.start))
}

fn variants(
    facts: &Declarations,
    poll: ItemId,
    parameter: TypeId,
) -> Result<crate::sema::members::VariantId, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("poll variants lost their actual catalog")?;
    let ready = catalog
        .validated_language(LangItem::PollReady)?
        .ok_or("poll operation requires actual Ready")?;
    let pending = catalog
        .validated_language(LangItem::PollPending)?
        .ok_or("poll operation requires actual Pending")?;
    let ready_id = variant(facts, ready, poll)?;
    let pending_id = variant(facts, pending, poll)?;
    if ready_id == pending_id
        || facts.members.variants(poll).len() != 2
        || !matches!(facts.members.fields(ready_id),Some([field]) if field.ty == parameter)
        || !matches!(facts.members.fields(pending_id), Some([]))
        || facts.members.variants[ready_id.0 as usize].shape != crate::syntax::item::Shape::Tuple
        || facts.members.variants[pending_id.0 as usize].shape != crate::syntax::item::Shape::Unit
    {
        return Err(
            "demanded poll requires exact actual Ready(Output) and unit Pending constructors",
        );
    }
    Ok(ready_id)
}

fn variant(
    facts: &Declarations,
    declaration: Declaration,
    owner: ItemId,
) -> Result<crate::sema::members::VariantId, &'static str> {
    let id = facts
        .members
        .at_node(declaration.origin)
        .ok_or("poll variant lost its source producer")?;
    let member = facts
        .members
        .variants
        .get(id.0 as usize)
        .ok_or("poll variant lost its actual member")?;
    // Reservation parents identify trait/impl scopes, not enum ownership.
    // The canonical member row is the actual enum-owner/ordinal authority.
    if declaration.parent.is_some()
        || member.owner != owner
        || member.origin.source() != Some(declaration.origin)
        || facts.members.variants(owner).get(member.ordinal as usize) != Some(&id)
    {
        return Err("poll variant differs from its actual source owner and ordinal");
    }
    Ok(id)
}

fn signature(
    facts: &Declarations,
    ty: TypeId,
    self_type: TypeId,
    pin: ItemId,
    poll: ItemId,
    context: ItemId,
    output: DefId,
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions: 3,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(ty)
    else {
        return Err(
            "demanded poll requires its actual safe Rust signature with three public binder slots",
        );
    };
    let [receiver, cx] = facts
        .types
        .list(parameters)
        .ok_or("poll signature lost its actual parameters")?
    else {
        return Err("demanded poll requires exactly its Pin receiver and Context input");
    };
    let Some(Kind::Adt(actual_pin, receiver_arguments)) = facts.types.kind(*receiver) else {
        return Err("poll receiver is not actual Pin");
    };
    let Some(Kind::Reference {
        referent: actual_context,
        mutable: true,
        region: cx_region,
    }) = facts.types.kind(*cx)
    else {
        return Err("poll Context input is not its actual mutable reference");
    };
    let Some(Kind::Adt(actual_context, context_arguments)) = facts.types.kind(actual_context)
    else {
        return Err("poll Context input lost its actual nominal");
    };
    let Some(Kind::Adt(actual_poll, result_arguments)) = facts.types.kind(result) else {
        return Err("poll result is not actual Poll");
    };
    let receiver = one_argument(facts, receiver_arguments)?;
    let context_argument = one_argument(facts, context_arguments)?;
    let result_argument = one_argument(facts, result_arguments)?;
    if actual_pin != pin
        || actual_context != context
        || actual_poll != poll
        || cx_region != RegionId::bound(0, 1)?
        || context_argument != Argument::Lifetime(RegionId::bound(0, 2)?)
        || !matches!(receiver,Argument::Type(ty) if matches!(facts.types.kind(ty),Some(Kind::Reference {referent,mutable:true,region}) if referent == self_type && region == RegionId::bound(0,0)?))
    {
        return Err(
            "poll public receiver/Context binder coordinates differ from the actual protocol",
        );
    }
    let Argument::Type(projected) = result_argument else {
        return Err("poll result lost its actual Output application");
    };
    let Some(Kind::Projection(application)) = facts.types.kind(projected) else {
        return Err("poll result lost its actual Output projection");
    };
    let (future, _) = super::schema::require(facts)?;
    let parent = facts
        .types
        .trait_application(application.trait_application)
        .ok_or("poll result lost its actual Future application")?;
    if application.subject != self_type
        || application.item != ItemId(output.0)
        || application.arguments != GenericArgs::EMPTY
        || parent.item != ItemId(future.0)
        || parent.arguments != GenericArgs::EMPTY
    {
        return Err("poll result differs from its actual Self/Future/Output relation");
    }
    Ok(())
}

fn one_argument(facts: &Declarations, arguments: GenericArgs) -> Result<Argument, &'static str> {
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("poll contract lost its complete arguments")?;
    if arguments.len() != 1 {
        return Err("poll contract requires exactly its declared argument");
    }
    arguments
        .next()
        .ok_or("poll contract lost its exact argument")
}

fn public_signature(
    facts: &mut Declarations,
    method: DefId,
    source: TypeId,
) -> Result<TypeId, &'static str> {
    let item = ItemId(method.0);
    let callable = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("poll lost its actual public callable template")?;
    let signature = callable.signature;
    if callable.parameters != 0 {
        return Err("poll public contract has unsupported early parameters");
    }
    let Some(Kind::FunctionItem {
        item: actual,
        signature: declared,
        ..
    }) = facts
        .definitions
        .get(item.0 as usize)
        .copied()
        .flatten()
        .and_then(|ty| facts.types.kind(ty))
    else {
        return Err("poll public contract lost its actual function item declaration");
    };
    if actual != item || declared != signature {
        return Err("poll public signature differs from its actual declaration/template");
    }
    let scheme = facts
        .lifetimes
        .functions
        .get(&method)
        .ok_or("poll source-to-public frame is absent")?;
    facts.charge_caller_inputs(
        scheme.total as usize,
        (scheme.total as usize).saturating_mul(size_of::<RegionId>()),
    )?;
    let mut work = 0;
    let mapping = scheme.callable_mapping(&mut work)?;
    // The exact source declaration is mapped by its existing owning lifetime scheme.
    let mut fold = crate::types::substitute::Bound::new(mapping);
    let payments =
        crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
            .map_err(|_| "poll signature relocation lost its actual source payer")?;
    let mapped = fold.signature_charged(&mut facts.types, source, 3, &|steps, bytes| {
        payments.charge(steps, bytes)
    })?;
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(work)
        .saturating_add(fold.steps);
    if mapped != signature {
        return Err("poll source/public signature relocation changed its actual contract");
    }
    Ok(signature)
}
