//! Generated arithmetic and bounds guards select checked source panic functions.
use super::{Catalog, inhabited_properties::Plans};
use crate::{
    resolve::{expansion::Report, unit::NodeKey},
    sema::{
        model::Declarations,
        values::{Evaluator, Failure, LocationId},
    },
    support::{bindings::LangItem, hash::HashMap, retained::Growth},
    types::{Kind, Primitive, TypeId, callable::Convention, instance::Instance},
};

#[derive(Clone, Copy)]
pub(crate) struct Route {
    pub instance: Instance,
    pub signature: TypeId,
    pub tracked: bool,
}

#[derive(Default)]
pub(super) struct Routes {
    targets: HashMap<LangItem, Route>,
    locations: HashMap<NodeKey, LocationId>,
}

impl Catalog {
    fn arithmetic_guards(&self) -> Option<&Routes> {
        self.native_layouts.as_deref()?.arithmetic_guards.as_deref()
    }

    pub(crate) fn guard_route(&self, role: LangItem) -> Option<Route> {
        self.arithmetic_guards()?.targets.get(&role).copied()
    }

    pub(crate) fn guard_location(&self, site: NodeKey) -> Option<LocationId> {
        self.arithmetic_guards()?.locations.get(&site).copied()
    }
}

pub(crate) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    role: LangItem,
) -> Result<Option<Route>, String> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    if let Some(route) = catalog.guard_route(role) {
        return Ok(Some(route));
    }
    let Some(declaration) = catalog.validated_language(role)? else {
        return Ok(None);
    };
    if !super::guard_panics::ROLES.contains(&role) {
        return Err("source guard requested a different language role".into());
    }
    catalog.require_provider_report(report)?;
    let header = super::headers::functions::current(facts, declaration)?;
    let signature = header.signature;
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::NEVER,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Ok(None);
    };
    let bounds = [TypeId::primitive(Primitive::Usize); 2];
    let expected: &[TypeId] = if role == LangItem::PanicBoundsCheck {
        &bounds
    } else {
        &[]
    };
    if facts.types.list(parameters) != Some(expected)
        || header.universals != 0
        || facts
            .generics
            .parameters_in_scope(declaration.definition)
            .next()
            .is_some()
        || facts
            .generics
            .declared_predicates_in_scope(declaration.definition)
            .next()
            .is_some()
    {
        return Ok(None);
    }
    let ty = facts.definitions[declaration.definition.0 as usize]
        .ok_or("source panic guard lost its canonical function item")?;
    let route = Route {
        instance: Instance::new(&facts.types, ty)?,
        signature,
        tracked: facts
            .checked_caller_tracking(report, declaration.definition)
            .map_err(|error| error.to_string())?,
    };
    let (len, capacity, routes) = catalog
        .arithmetic_guards()
        .map_or((0, 0, size_of::<Routes>()), |routes| {
            (routes.targets.len(), routes.targets.capacity(), 0)
        });
    let owner = usize::from(catalog.native_layouts.is_none()) * size_of::<Plans>();
    let growth = Growth::map::<LangItem, Route>(len, capacity, 1);
    facts.charge_caller_inputs(
        growth.work.saturating_add(1),
        growth.bytes.saturating_add(routes).saturating_add(owner),
    )?;
    let targets = &mut facts
        .source_bindings
        .as_deref_mut()
        .ok_or("source guard lost its authenticated catalog")?
        .native_layouts
        .get_or_insert_with(Default::default)
        .arithmetic_guards
        .get_or_insert_with(Default::default)
        .targets;
    targets
        .try_reserve(1)
        .map_err(|_| "source guard target allocation failed")?;
    targets.insert(role, route);
    Ok(Some(route))
}

pub(crate) fn prepare_location(
    report: &Report,
    facts: &mut Declarations,
    site: NodeKey,
) -> Result<(), String> {
    let routes = facts
        .source_bindings
        .as_deref()
        .and_then(Catalog::arithmetic_guards)
        .ok_or("source guard location requires its checked target")?;
    if routes.locations.contains_key(&site) {
        return Ok(());
    }
    let growth =
        Growth::map::<NodeKey, LocationId>(routes.locations.len(), routes.locations.capacity(), 1);
    facts.charge_caller_inputs(growth.work.saturating_add(1), growth.bytes)?;
    let location = Evaluator::new(report, facts)
        .prepare_source_location(site)
        .map_err(Failure::message)?;
    let locations = &mut facts
        .source_bindings
        .as_deref_mut()
        .and_then(|catalog| catalog.native_layouts.as_deref_mut())
        .and_then(|plans| plans.arithmetic_guards.as_deref_mut())
        .ok_or("source guard lost its checked target")?
        .locations;
    locations
        .try_reserve(1)
        .map_err(|_| "source guard location allocation failed")?;
    locations.insert(site, location);
    Ok(())
}
