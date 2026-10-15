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
        substitute::Bound,
    },
};
use std::collections::HashMap;

pub(crate) struct Dispatcher {
    normalize: Bound,
    context: Context,
    obligations: Obligations,
    selected: HashMap<TypeId, TypeId>,
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
        })
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
                assumptions: &empty,
                environment: None,
                first_region: self.obligations.regions.end,
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
    pub(crate) fn finish(mut self) -> Obligations {
        self.obligations
            .equalities
            .append(&mut self.context.region_equalities);
        self.obligations
    }
}
