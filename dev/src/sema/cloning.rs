//! Canonical Clone members retain marker obligations separately from executable impls.
use crate::{
    resolve::model::{BuiltinCloneImplementation, BuiltinCloning, DefId},
    types::{TypeId, parameters::predicates::Predicate},
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Original {
    pub declaration: BuiltinCloneImplementation,
    pub parameter: TypeId,
    pub subject: TypeId,
    pub signature: TypeId,
    pub predicate: Predicate,
}
pub(crate) struct Registry {
    pub canonical: BuiltinCloning,
    pub original: Option<Original>,
    pub complete: bool,
    pub requested: bool,
}
impl Registry {
    pub fn owns_trait(&self, definition: DefId) -> bool {
        self.canonical.definition == definition
    }
}
impl Original {
    pub fn validate(self, facts: &super::model::Declarations) -> Result<(), &'static str> {
        use crate::types::{
            GenericArgs, ItemId, Kind, RegionId, arguments::Argument, parameters::ParameterKind,
        };
        let registry = facts.cloning.as_deref().ok_or("missing Clone registry")?;
        if !registry.complete
            || registry.original != Some(self)
            || registry.canonical.implementation != Some(self.declaration)
        {
            return Err("iterator Clone original lacks complete registered conformance");
        }
        let implementations = facts
            .generics
            .implementations
            .as_ref()
            .ok_or("missing Clone implementation inventory")?;
        if !implementations.is_validated(self.declaration.definition)
            || implementations
                .header(self.declaration.definition)
                .is_none_or(|h| h.predicate != self.predicate)
            || implementations.method(self.declaration.definition, registry.canonical.method)
                != Some(self.declaration.method)
        {
            return Err("iterator Clone original differs from selected implementation");
        }
        let Some(Kind::Parameter(parameter)) = facts.types.kind(self.parameter) else {
            return Err("invalid iterator Clone parameter");
        };
        if facts.types.parameter(parameter).is_none_or(|p| {
            p.owner != ItemId(self.declaration.definition.0)
                || p.index != 1
                || p.kind != (ParameterKind::Type { sized: true })
        }) {
            return Err("iterator Clone parameter differs from registered owner");
        }
        let iterator = facts
            .iteration
            .as_deref()
            .and_then(|r| r.collections.as_deref())
            .ok_or("missing canonical iterator collection")?
            .canonical
            .iterator;
        let Some(Kind::Adt(owner, arguments)) = facts.types.kind(self.subject) else {
            return Err("iterator Clone subject is not nominal");
        };
        let lifetime = RegionId::bound(0, 0)?;
        if owner != ItemId(iterator.0)
            || facts.types.arguments(arguments).is_none_or(|a| {
                a.collect::<Vec<_>>()
                    != [Argument::Lifetime(lifetime), Argument::Type(self.parameter)]
            })
        {
            return Err("iterator Clone subject differs from canonical schema");
        }
        self.validate_signature(facts)?;
        let Some((application, _)) = self.predicate.bound.trait_parts() else {
            return Err("iterator Clone requires a real trait application");
        };
        let application = facts
            .types
            .trait_application(application)
            .ok_or("missing iterator Clone application")?;
        if self.predicate.ty != self.subject
            || application.item != ItemId(registry.canonical.definition.0)
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.count() != 0)
            || facts.definitions[self.declaration.method.0 as usize]
                .and_then(|ty| facts.types.kind(ty))
                != Some(Kind::FunctionItem {
                    item: ItemId(self.declaration.method.0),
                    signature: self.signature,
                    arguments: GenericArgs::EMPTY,
                })
        {
            return Err("iterator Clone implementation identity differs from callable");
        }
        Ok(())
    }
    fn validate_signature(self, facts: &super::model::Declarations) -> Result<(), &'static str> {
        use crate::types::{Kind, RegionId};
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
            bound_regions: 2,
        }) = facts.types.kind(self.signature)
        else {
            return Err("invalid iterator Clone callable contract");
        };
        let [receiver] = facts
            .types
            .list(parameters)
            .ok_or("missing Clone parameters")?
        else {
            return Err("invalid iterator Clone arity");
        };
        if result != self.subject
            || facts.types.kind(*receiver)
                != Some(Kind::Reference {
                    referent: self.subject,
                    mutable: false,
                    region: RegionId::bound(0, 1)?,
                })
        {
            return Err("iterator Clone receiver/result differs from exact signature");
        }
        let scheme = facts
            .lifetimes
            .functions
            .get(&self.declaration.method)
            .ok_or("missing iterator Clone lifetime scheme")?;
        if scheme.total != 2
            || scheme.captures != 1
            || !scheme.early.is_empty()
            || scheme.late != [1]
            || !scheme.bounds.is_empty()
        {
            return Err("iterator Clone lifetime binder differs from registered scheme");
        }
        Ok(())
    }
}
/// Only the prepared canonical Clone application has this marker counterpart.
/// This supplies abstract checking, never an implementation or callable target.
pub(in crate::sema) fn marker_predicate(
    facts: &super::model::Declarations,
    predicate: Predicate,
) -> Option<Predicate> {
    let registry = facts.cloning.as_deref()?;
    let (application, _) = predicate.bound.trait_parts()?;
    let application = facts.types.trait_application(application)?;
    if application.item.0 != registry.canonical.definition.0
        || facts.types.arguments(application.arguments)?.count() != 0
    {
        return None;
    }
    Some(Predicate {
        ty: predicate.ty,
        bound: crate::types::parameters::predicates::Bound::Clone,
    })
}
