//! Trait and implementation method arguments share declaration order, not parameter identities.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        GenericArgs, ItemId, Kind, RegionId, TypeId,
        arguments::Argument,
        parameters::{
            ParameterId,
            predicates::{Bound, Predicate},
        },
        substitute,
        traits::Application,
    },
};

pub(super) struct Arguments {
    method: DefId,
    regions: Vec<RegionId>,
    bindings: Vec<(ParameterId, Argument)>,
}
impl Arguments {
    pub(super) fn substitution(
        &self,
        facts: &Declarations,
    ) -> Result<substitute::Bound, &'static str> {
        substitute::Bound::parameters_captured(&facts.types, &self.bindings, self.regions.clone())
    }
    pub(super) fn new(facts: &Declarations, ty: TypeId) -> Result<Self, &'static str> {
        let Some(Kind::FunctionItem {
            item, arguments, ..
        }) = facts.types.kind(ty)
        else {
            return Err("missing trait callable identity");
        };
        let template = facts
            .lifetimes
            .callables
            .get(item)
            .ok_or("missing trait callable template")?;
        let parameters = facts
            .lifetimes
            .callables
            .generic_parameters(item)
            .ok_or("missing trait callable parameters")?;
        let arguments = facts
            .types
            .arguments(arguments)
            .ok_or("invalid trait callable arguments")?
            .collect::<Vec<_>>();
        let early = template.parameters as usize;
        if arguments.len() != early.saturating_add(parameters.len()) {
            return Err("trait callable argument arity mismatch");
        }
        let regions = arguments[..early]
            .iter()
            .map(|argument| {
                if let Argument::Lifetime(region) = *argument {
                    Ok(region)
                } else {
                    Err("trait callable early argument is not a lifetime")
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            method: DefId(item.0),
            regions,
            bindings: parameters
                .iter()
                .copied()
                .zip(arguments[early..].iter().copied())
                .collect(),
        })
    }
    pub(super) fn constant(
        facts: &Declarations,
        instance: crate::sema::values::Instance,
    ) -> Result<Self, &'static str> {
        let captures = facts
            .associated
            .schema(instance.definition)
            .ok_or("missing constant owner schema")?
            .captures as usize;
        let (_, parameters) = facts
            .generics
            .callable_parameters(&facts.types, instance.definition)?;
        let arguments = facts
            .types
            .arguments(instance.arguments)
            .ok_or("invalid constant owner arguments")?
            .collect::<Vec<_>>();
        if arguments.len() != captures.saturating_add(parameters.len()) {
            return Err("constant owner argument arity mismatch");
        }
        let regions = arguments[..captures]
            .iter()
            .map(|argument| match argument {
                Argument::Lifetime(region) => Ok(*region),
                _ => Err("invalid constant lifetime argument"),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            method: instance.definition,
            regions,
            bindings: parameters
                .into_iter()
                .zip(arguments[captures..].iter().copied())
                .collect(),
        })
    }
    pub(super) fn goal(
        &self,
        facts: &mut Declarations,
        owner: DefId,
    ) -> Result<Predicate, &'static str> {
        let subject = facts
            .associated
            .schema(owner)
            .ok_or("missing dispatch trait schema")?
            .self_type;
        let arguments = facts
            .generics
            .parameters
            .get(&owner)
            .ok_or("missing dispatch trait parameters")?
            .iter()
            .map(|p| p.argument)
            .collect::<Vec<_>>();
        let arguments = facts.types.intern_arguments(&arguments)?;
        let application = facts.types.intern_trait_application(Application {
            item: ItemId(owner.0),
            arguments,
        })?;
        let mut fold = substitute::Bound::parameters_captured(
            &facts.types,
            &self.bindings,
            self.regions.clone(),
        )?;
        fold.predicate(
            &mut facts.types,
            Predicate {
                ty: subject,
                bound: Bound::Trait(application),
            },
        )
    }
    pub(super) fn target(
        &self,
        facts: &mut Declarations,
        implementation: DefId,
        selected: GenericArgs,
        method: DefId,
    ) -> Result<TypeId, &'static str> {
        let captures = facts
            .associated
            .schema(implementation)
            .ok_or("missing selected impl schema")?
            .captures as usize;
        let actual = facts
            .types
            .arguments(selected)
            .ok_or("invalid selected impl arguments")?
            .collect::<Vec<_>>();
        if actual.len() < captures {
            return Err("incomplete selected impl lifetime arguments");
        }
        let source_captures = facts
            .lifetimes
            .functions
            .get(&self.method)
            .ok_or("missing trait method lifetime scheme")?
            .captures as usize;
        let own_regions = self
            .regions
            .get(source_captures..)
            .ok_or("missing trait method early arguments")?;
        let mut arguments = actual[..captures].to_vec();
        arguments.extend(own_regions.iter().copied().map(Argument::Lifetime));
        arguments.extend_from_slice(&actual[captures..]);
        for &(id, argument) in &self.bindings {
            if facts
                .types
                .parameter(id)
                .ok_or("invalid trait method parameter")?
                .owner
                == ItemId(self.method.0)
            {
                arguments.push(argument);
            }
        }
        instantiate(facts, method, &arguments)
    }
}

pub(super) fn instantiate(
    facts: &mut Declarations,
    method: DefId,
    arguments: &[Argument],
) -> Result<TypeId, &'static str> {
    let item = ItemId(method.0);
    let signature = if let Some(template) = facts.lifetimes.callables.get(item) {
        let early = template.parameters as usize;
        let parameters = facts
            .lifetimes
            .callables
            .generic_parameters(item)
            .ok_or("missing implementation method parameters")?;
        if arguments.len() != early.saturating_add(parameters.len()) {
            return Err("implementation method argument arity mismatch");
        }
        let regions = arguments[..early]
            .iter()
            .map(|arg| {
                if let Argument::Lifetime(region) = *arg {
                    Ok(region)
                } else {
                    Err("invalid implementation method lifetime argument")
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let bindings = parameters
            .iter()
            .copied()
            .zip(arguments[early..].iter().copied())
            .collect::<Vec<_>>();
        let signature = template.signature;
        substitute::Bound::parameters_captured(&facts.types, &bindings, regions)?
            .apply(&mut facts.types, signature)?
    } else {
        if !arguments.is_empty() {
            return Err("implementation method lacks its generic template");
        }
        ordinary_signature(facts, method)?
    };
    let arguments = facts.types.intern_arguments(arguments)?;
    facts.types.intern(Kind::FunctionItem {
        item,
        arguments,
        signature,
    })
}

fn ordinary_signature(facts: &mut Declarations, method: DefId) -> Result<TypeId, &'static str> {
    let ty = facts
        .definitions
        .get(method.0 as usize)
        .copied()
        .flatten()
        .ok_or("missing implementation method type")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(ty) else {
        return Err("implementation method is not callable");
    };
    let Some(scheme) = facts.lifetimes.functions.get(&method) else {
        return Ok(signature);
    };
    if scheme.captures != 0 || !scheme.early.is_empty() {
        return Err("implementation method requires a captured template");
    }
    let mut regions = vec![RegionId::STATIC; scheme.total as usize];
    for (index, &slot) in scheme.late.iter().enumerate() {
        regions[usize::from(slot)] = RegionId::bound(
            0,
            u16::try_from(index).map_err(|_| "too many method late lifetimes")?,
        )?;
    }
    let late = u32::try_from(scheme.late.len()).map_err(|_| "too many method late lifetimes")?;
    substitute::Bound::new(regions).signature(&mut facts.types, signature, late)
}
