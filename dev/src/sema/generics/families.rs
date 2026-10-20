//! A selected associated definition substitutes parent and family arguments together.
use super::{DefId, Parameter, parameter_id};
use crate::{
    sema::model::Declarations,
    types::{
        GenericArgs, RegionId, Store, TypeId,
        arguments::Argument,
        infer::{Context, TermId, arguments::Argument as Inferred},
        parameters::ParameterId,
        substitute::{Bound, families as family},
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
        family::member_mappings(
            &self.types,
            crate::types::ItemId(member.0),
            parameters.iter().map(|p| p.argument),
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

fn mappings(
    store: &Store,
    application: Application,
    (parent, own): (&[Parameter], &[Parameter]),
    captures: u32,
) -> Result<family::Mappings, &'static str> {
    family::mappings(
        store,
        family::Application {
            owner: crate::types::ItemId(application.owner.0),
            parent: application.parent,
            member: crate::types::ItemId(application.member.0),
            own: application.own,
        },
        (
            parent.iter().map(|p| p.argument),
            own.iter().map(|p| p.argument),
        ),
        captures,
    )
}

#[cfg(test)]
#[path = "../../../tests/invariants/family_arguments.rs"]
mod tests;
