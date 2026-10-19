//! A selected associated definition substitutes parent and family arguments together.
use super::{DefId, Parameter, parameter_id};
use crate::{
    sema::model::Declarations,
    types::{
        GenericArgs, RegionId, Store, TypeId,
        arguments::Argument,
        infer::{Context, TermId, arguments::Argument as Inferred},
        parameters::ParameterId,
        substitute::Bound,
    },
};
use std::collections::{HashMap, hash_map::Entry};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Application {
    owner: DefId,
    parent: GenericArgs,
    member: DefId,
    own: GenericArgs,
}

#[derive(Default)]
pub(super) struct Substitutions {
    completed: HashMap<Application, Bound>,
}

impl Declarations {
    /// Map a declared trait contract, optionally including one member's own inputs.
    /// All replacements are simultaneous; none of these clauses grants caller proof.
    pub(in crate::sema) fn trait_substitution(
        &self,
        predicate: crate::types::parameters::predicates::Predicate,
        family: Option<(DefId, GenericArgs)>,
    ) -> Result<Bound, &'static str> {
        let crate::types::parameters::predicates::Bound::Trait(id) = predicate.bound else {
            return Err("missing associated result trait");
        };
        let parent = self
            .types
            .trait_application(id)
            .ok_or("missing associated result application")?;
        let owner = DefId(parent.item.0);
        let schema = self
            .associated
            .schema(owner)
            .ok_or("missing associated result schema")?;
        let (member, own) = family.unwrap_or((owner, GenericArgs::EMPTY));
        if family.is_some() && self.associated.trait_owner(member) != Some(owner) {
            return Err("associated family does not belong to the declared trait");
        }
        let parameters = self
            .generics
            .parameters
            .get(&owner)
            .map_or(&[][..], Vec::as_slice);
        let own_parameters = family
            .and_then(|_| self.generics.parameters.get(&member))
            .map_or(&[][..], Vec::as_slice);
        let mut inputs = mappings(
            &self.types,
            Application {
                owner,
                parent: parent.arguments,
                member,
                own,
            },
            (parameters, own_parameters),
            schema.captures,
        )?;
        inputs.parameters.push((
            parameter_id(&self.types, Argument::Type(schema.self_type))?,
            Argument::Type(predicate.ty),
        ));
        inputs.finish(&self.types)
    }

    /// Expose a selected definition's shape with the caller's still-inferred inputs.
    /// Both owners share one substitution; inserted arguments are never remapped.
    /// This supplies no formation or implementation proof. The body must retain
    /// the original projection and discharge all of its input requirements.
    pub(in crate::sema) fn infer_associated_definition(
        &mut self,
        context: &mut Context,
        member: DefId,
        own: &[Inferred],
        mut bindings: Vec<(ParameterId, Inferred)>,
        mut regions: Vec<RegionId>,
    ) -> Result<TermId, String> {
        let parameters = self
            .generics
            .parameters
            .get(&member)
            .map_or(&[][..], Vec::as_slice);
        if self
            .associated
            .families
            .get(&member)
            .is_some_and(|f| f.captures as usize != regions.len())
        {
            return Err("associated definition is missing parent lifetime captures".into());
        }
        context
            .account_work(parameters.len())
            .map_err(|e| e.to_string())?;
        member_mappings(
            &self.types,
            member,
            parameters,
            own.iter().copied(),
            &mut bindings,
            &mut regions,
            |arg| match arg {
                Inferred::Lifetime(region) => Some(region),
                Inferred::Type(_) | Inferred::Const(_) => None,
            },
        )?;
        let template = self
            .definitions
            .get(member.0 as usize)
            .copied()
            .flatten()
            .ok_or("missing associated result definition")?;
        let mut fold = Bound::parameters_captured(&self.types, &[], regions)?;
        let template = fold.apply(&mut self.types, template)?;
        context
            .account_work(fold.steps)
            .map_err(|e| e.to_string())?;
        context
            .instantiate(&mut self.types, template, &bindings)
            .map_err(|e| e.to_string())
    }

    /// Structural substitution only: selection and family formation must already
    /// justify this exact member application. No proof is cached with its folder.
    pub(in crate::sema) fn instantiate_associated_definition(
        &mut self,
        owner: DefId,
        parent: GenericArgs,
        member: DefId,
        own: GenericArgs,
        template: TypeId,
    ) -> Result<TypeId, &'static str> {
        let parameters = self
            .generics
            .parameters
            .get(&member)
            .map_or(&[][..], Vec::as_slice);
        if own == GenericArgs::EMPTY && parameters.is_empty() {
            return self
                .generics
                .substitute(&mut self.types, owner, parent, template);
        }
        if self.associated.type_definition(owner, member) != Some(member) {
            return Err("associated definition does not belong to the selected owner");
        }
        let captures = self
            .associated
            .schema(owner)
            .ok_or("missing associated definition owner schema")?
            .captures;
        let application = Application {
            owner,
            parent,
            member,
            own,
        };
        let parents = self
            .generics
            .parameters
            .get(&owner)
            .map_or(&[][..], Vec::as_slice);
        let substitutions = self
            .generics
            .family_substitutions
            .get_or_insert_with(Default::default);
        let fold = match substitutions.completed.entry(application) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(joint(
                &self.types,
                application,
                (parents, parameters),
                captures,
            )?),
        };
        fold.apply(&mut self.types, template)
    }
}

fn joint(
    store: &Store,
    application: Application,
    parameters: (&[Parameter], &[Parameter]),
    captures: u32,
) -> Result<Bound, &'static str> {
    mappings(store, application, parameters, captures)?.finish(store)
}

struct Mappings {
    parameters: Vec<(ParameterId, Argument)>,
    regions: Vec<RegionId>,
}
impl Mappings {
    fn finish(self, store: &Store) -> Result<Bound, &'static str> {
        Bound::parameters_captured(store, &self.parameters, self.regions)
    }
}

fn mappings(
    store: &Store,
    application: Application,
    parameters: (&[Parameter], &[Parameter]),
    captures: u32,
) -> Result<Mappings, &'static str> {
    let (parent, own) = parameters;
    let mut parent_arguments = store
        .arguments(application.parent)
        .ok_or("invalid associated parent arguments")?;
    let own_arguments = store
        .arguments(application.own)
        .ok_or("invalid associated family arguments")?;
    let captures = captures as usize;
    let parent_parameters = parent
        .iter()
        .filter(|p| !matches!(p.argument, Argument::Lifetime(_)));
    if parent_arguments.len() != captures + parent_parameters.clone().count()
        || own_arguments.len() != own.len()
        || parent_arguments.len().saturating_add(own_arguments.len()) > 1_000_000
    {
        return Err("associated family argument arity or storage limit exceeded");
    }
    let mut regions = Vec::with_capacity(captures);
    for _ in 0..captures {
        let Some(Argument::Lifetime(region)) = parent_arguments.next() else {
            return Err("associated parent requires its complete lifetime captures");
        };
        regions.push(region);
    }
    let mut mappings = Vec::new();
    for (formal, actual) in parent_parameters.zip(parent_arguments) {
        let parameter = parameter_id(store, formal.argument)?;
        if store
            .parameter(parameter)
            .is_none_or(|p| p.owner.0 != application.owner.0)
        {
            return Err("associated parent parameter belongs to another owner");
        }
        mappings.push((parameter, actual));
    }
    member_mappings(
        store,
        application.member,
        own,
        own_arguments,
        &mut mappings,
        &mut regions,
        |arg| match arg {
            Argument::Lifetime(region) => Some(region),
            Argument::Type(_) | Argument::Const(_) => None,
        },
    )?;
    Ok(Mappings {
        parameters: mappings,
        regions,
    })
}

// Canonical normalization and inferred result shapes use the same declaration
// identities, complete capture prefix, arity and lifetime-slot validation.
fn member_mappings<A: Copy>(
    store: &Store,
    member: DefId,
    parameters: &[Parameter],
    arguments: impl ExactSizeIterator<Item = A>,
    mappings: &mut Vec<(ParameterId, A)>,
    regions: &mut Vec<RegionId>,
    lifetime: impl Fn(A) -> Option<RegionId>,
) -> Result<(), &'static str> {
    if parameters.len() != arguments.len() || parameters.len() > 1_000_000 {
        return Err("associated family argument arity or storage limit exceeded");
    }
    for (formal, actual) in parameters.iter().zip(arguments) {
        if let Argument::Lifetime(region) = formal.argument {
            let slot =
                u16::try_from(regions.len()).map_err(|_| "family lifetime capacity exceeded")?;
            if region != RegionId::bound(0, slot)? {
                return Err("associated family lifetime is outside its captured binder");
            }
            regions
                .push(lifetime(actual).ok_or("associated family lifetime argument kind mismatch")?);
        } else {
            let parameter = parameter_id(store, formal.argument)?;
            if store
                .parameter(parameter)
                .is_none_or(|p| p.owner.0 != member.0)
            {
                return Err("associated family parameter belongs to another member");
            }
            mappings.push((parameter, actual));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/family_arguments.rs"]
mod tests;
