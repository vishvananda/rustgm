//! Concrete trait callables select checked overrides while retaining their lifetime proof.
mod arguments;
pub(crate) use super::traits::obligations::Obligations;
use super::{
    model::Declarations,
    traits::{assumptions::Assumptions, selection},
};
use crate::{
    resolve::model::DefId,
    types::{
        Kind, RegionId, TypeId,
        infer::{Context, Instantiation},
        parameters::environment::Environment,
        substitute::Bound,
    },
};
use std::collections::HashMap;
use std::sync::Arc;

struct Caller {
    environment: Option<Arc<Environment>>,
    assumptions: Assumptions,
}

pub(crate) struct Dispatcher {
    normalize: Bound,
    context: Context,
    obligations: Obligations,
    selected: HashMap<TypeId, TypeId>,
    constants: HashMap<super::values::Instance, super::values::Instance>,
    expanded: HashMap<super::values::Instance, bool>,
    caller: Option<Box<Caller>>,
}
impl Dispatcher {
    pub(crate) fn new(universals: u32, first: u32) -> Result<Self, &'static str> {
        let end = first
            .checked_add(universals)
            .filter(|&end| end < 0x8000_0000)
            .ok_or("dispatch region identity space exhausted")?;
        if first == 0 {
            return Err("dispatch requires a free region allocator");
        }
        let mut obligations = Obligations::new(first);
        obligations.regions.end = end;
        let regions = (first..end).map(RegionId).collect::<Vec<_>>();
        for (index, &region) in regions.iter().enumerate() {
            let slot = u16::try_from(index).map_err(|_| "too many dispatch universals")?;
            obligations
                .equalities
                .push((region, RegionId::bound(0, slot)?));
        }
        Ok(Self {
            normalize: Bound::new(regions),
            context: Context::default(),
            obligations,
            selected: HashMap::new(),
            constants: HashMap::new(),
            expanded: HashMap::new(),
            caller: None,
        })
    }
    /// Patterns must select a definition before monomorphization. Rigid caller
    /// parameters may remain in unused arguments, but an assumption is not a value.
    pub(crate) fn pattern(
        facts: &mut Declarations,
        universals: u32,
        first: u32,
        owner: Option<DefId>,
    ) -> Result<Self, String> {
        let mut result = Self::new(universals, first)?;
        let environment = owner
            .and_then(|owner| facts.generics.environments.get(&owner))
            .cloned()
            .map(|environment| {
                environment
                    .substitute_regions(&mut facts.types, &mut result.normalize)
                    .map(Arc::new)
            })
            .transpose()?;
        result
            .context
            .account_work(result.normalize.steps)
            .map_err(|e| e.to_string())?;
        let mut assumptions = Assumptions::default();
        for predicate in environment.iter().flat_map(|env| env.trait_predicates()) {
            result.context.account_work(1).map_err(|e| e.to_string())?;
            // Only a fully global bound can use its known implementation.
            // Free lifetimes include 'static; either that input or a parameter
            // keeps the caller assumption opaque to constant-value selection.
            if predicate.contains_parameters(&facts.types)?
                || predicate.has_free_regions(&facts.types)?
            {
                assumptions
                    .insert(&facts.types, *predicate)
                    .map_err(|e| e.to_string())?;
            }
        }
        result.caller = Some(Box::new(Caller {
            environment,
            assumptions,
        }));
        Ok(result)
    }
    pub(crate) fn select(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<TypeId, String> {
        if let Some(&selected) = self.selected.get(&ty) {
            return Ok(selected);
        }
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(ty) else {
            return Err("dispatch requires a function item".into());
        };
        let method = DefId(item.0);
        let Some(trait_owner) = facts.associated.trait_owner(method) else {
            return Ok(ty);
        };
        if facts.types.contains_parameters(ty) != Some(false) {
            return Err("concrete dispatch retains unresolved parameters".into());
        }
        let arguments = arguments::Arguments::new(facts, ty)?;
        let predicate = arguments.goal(facts, trait_owner)?;
        let (owner, implementation, evidence) = self.implementation(facts, predicate)?;
        let target = facts
            .generics
            .implementations
            .as_ref()
            .and_then(|registry| registry.method(owner, method))
            .ok_or("selected implementation lacks the trait method")?;
        let result = if target == method {
            ty
        } else {
            arguments.target(facts, owner, implementation, target)?
        };
        self.obligations.append(evidence)?;
        self.selected.insert(ty, result);
        Ok(result)
    }
    pub(crate) fn constant(
        &mut self,
        facts: &mut Declarations,
        instance: super::values::Instance,
    ) -> Result<super::values::Instance, String> {
        let selected = self.select_constant(facts, instance)?;
        facts.constant_targets.insert(instance, selected);
        let mut work = vec![(selected, false)];
        while let Some((current, finish)) = work.pop() {
            self.context.account_work(1).map_err(|e| e.to_string())?;
            if finish {
                self.expanded.insert(current, true);
                continue;
            }
            match self.expanded.get(&current) {
                Some(true) => continue,
                Some(false) => return Err("cycle in associated constant dependencies".into()),
                None => {}
            }
            self.expanded.insert(current, false);
            if self.expanded.len() > 1_000_000 {
                return Err("constant selection dependency budget exceeded".into());
            }
            work.push((current, true));
            let Some(dependencies) = facts
                .constant_dependencies
                .get(&current.definition)
                .cloned()
            else {
                continue;
            };
            self.context
                .account_work(dependencies.len())
                .map_err(|e| e.to_string())?;
            let mut substitution = current.substitution(facts)?;
            for dependency in dependencies.into_iter().rev() {
                let before = substitution.steps;
                match dependency {
                    super::values::Dependency::Constant(instance) => {
                        let source = instance.substitute(&mut facts.types, &mut substitution)?;
                        let target = self.select_constant(facts, source)?;
                        facts.constant_targets.insert(source, target);
                        work.push((target, false));
                    }
                    super::values::Dependency::Callable(ty) => {
                        let source = substitution.apply(&mut facts.types, ty)?;
                        let target = self.select(facts, source)?;
                        facts
                            .generics
                            .constant_callable_targets
                            .insert(source, target);
                    }
                }
                self.context
                    .account_work(substitution.steps - before)
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(selected)
    }
    fn select_constant(
        &mut self,
        facts: &mut Declarations,
        instance: super::values::Instance,
    ) -> Result<super::values::Instance, String> {
        if let Some(&selected) = self.constants.get(&instance) {
            return Ok(selected);
        }
        let Some(owner) = facts.associated.trait_owner(instance.definition) else {
            return Ok(instance);
        };
        let arguments = arguments::Arguments::constant(facts, instance)?;
        let predicate = arguments.goal(facts, owner)?;
        if self.caller.is_none() && predicate.contains_parameters(&facts.types)? {
            return Err("concrete constant retains unresolved arguments".into());
        }
        let (owner, arguments, evidence) = self.implementation(facts, predicate)?;
        let definition = facts
            .generics
            .implementations
            .as_ref()
            .and_then(|registry| registry.constant(owner, instance.definition))
            .ok_or("selected implementation lacks the trait constant")?;
        let result = if definition == instance.definition {
            instance
        } else {
            super::values::Instance {
                definition,
                arguments,
            }
        };
        self.obligations.append(evidence)?;
        self.constants.insert(instance, result);
        Ok(result)
    }
    fn implementation(
        &mut self,
        facts: &mut Declarations,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<(DefId, crate::types::GenericArgs, selection::Evidence), String> {
        let caller = self.caller.as_ref();
        let before = self.normalize.steps;
        let predicate = self.normalize.predicate(&mut facts.types, predicate)?;
        self.context
            .account_work(self.normalize.steps - before)
            .map_err(|e| e.to_string())?;
        let mut substitution =
            Instantiation::new(&mut self.context, &facts.types, &[]).map_err(|e| e.to_string())?;
        let goal = self
            .context
            .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
            .map_err(|e| e.to_string())?;
        let empty = Assumptions::default();
        let selected = selection::select(
            facts,
            &mut self.context,
            &goal,
            selection::Caller {
                assumptions: caller.map_or(&empty, |c| &c.assumptions),
                environment: caller.and_then(|c| c.environment.as_ref()),
                first_region: self.obligations.regions.end,
                retain_sized: false,
            },
        )?;
        let selection::Selection::Proven(evidence) = selected else {
            return Err("trait callable requires an unambiguous concrete implementation".into());
        };
        let selection::Source::Implementation {
            owner,
            arguments: implementation,
        } = evidence.source
        else {
            return Err("concrete dispatch cannot use a symbolic assumption".into());
        };
        Ok((owner, implementation, evidence))
    }
    pub(crate) fn finish(mut self) -> Obligations {
        self.obligations
            .equalities
            .append(&mut self.context.region_equalities);
        self.obligations
    }
}
