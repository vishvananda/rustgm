//! Alias substitution binds every actual formal, including an independent unsized Self.
use super::{Edge, Pending};
use crate::{
    sema::generics::Generics,
    types::{Store, Kind, arguments::{Argument, ConstKind}, parameters::ParameterId, substitute::Bound},
};
impl Pending {
    pub(super) fn mapping(&self, store: &Store, generics: &Generics, edge: Edge) -> Result<Bound, &'static str> {
        let header = self.header(edge.target)?;
        let schema = header.schema.as_ref().ok_or("alias schema is not prepared")?;
        if !schema.bounds.is_empty() {
            return Err("alias lifetime relations require a scoped region consumer");
        }
        let parameters = generics.parameters.get(&edge.target).ok_or("alias formals are not prepared")?;
        let arguments = store.arguments(edge.arguments).ok_or("invalid alias arguments")?;
        if parameters.len() != arguments.len() { return Err("incorrect canonical alias arity") }
        self.budget.charge(parameters.len().saturating_add(1),
            parameters.len().saturating_add(1).saturating_mul(size_of::<(ParameterId, Argument)>())
                .saturating_add((schema.captures as usize).saturating_mul(size_of::<crate::types::RegionId>())))?;
        let mut mappings = Vec::with_capacity(parameters.len().saturating_add(1));
        let mut regions = Vec::with_capacity(schema.captures as usize);
        let Kind::Parameter(self_parameter) = store.kind(schema.self_type).ok_or("invalid alias Self type")? else {
            return Err("alias Self is not its own rigid parameter");
        };
        mappings.push((self_parameter, Argument::Type(edge.subject)));
        for (parameter, argument) in parameters.iter().zip(arguments) {
            match parameter.argument {
                Argument::Lifetime(_) => {
                    let Argument::Lifetime(region) = argument else { return Err("alias lifetime argument has wrong kind") };
                    regions.push(region);
                }
                template => mappings.push((parameter_id(store, template)?, argument)),
            }
        }
        if regions.len() != schema.captures as usize { return Err("alias captured lifetime arity differs from its formals") }
        Bound::parameters_captured_charged(store, &mappings, regions, &|steps, bytes| self.budget.charge(steps, bytes))
    }
}
fn parameter_id(store: &Store, argument: Argument) -> Result<ParameterId, &'static str> {
    match argument {
        Argument::Type(ty) => match store.kind(ty) {
            Some(Kind::Parameter(id)) => Ok(id),
            _ => Err("alias type formal is not a rigid parameter"),
        },
        Argument::Const(id) => match store.const_kind(id) {
            Some(ConstKind::Parameter(id)) => Ok(id),
            _ => Err("alias const formal is not a rigid parameter"),
        },
        Argument::Lifetime(_) => Err("alias region cannot be used as a type/const formal"),
    }
}
