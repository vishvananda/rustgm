//! Fresh output witnesses transport identities while retaining every child obligation.
use super::{Checker, Error};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::unit::NodeKey,
    sema::{
        bodies::closures::universes::Interface,
        model::BodyRegionOrigin,
        nested_outputs::{Bridge, Formation, Link},
    },
    types::{RegionId, substitute::Bound},
};

pub(in crate::sema::bodies) struct Ports {
    pub pairs: HashMap<RegionId, RegionId>,
    parent: NodeKey,
}
impl Checker<'_> {
    pub(in crate::sema::bodies) fn retain_inline_root(&mut self) -> Result<(), Error> {
        let source = self.state.body;
        if self
            .facts
            .inline_constants
            .as_deref()
            .is_none_or(|registry| registry.get(source).is_none())
            || !self
                .facts
                .nested_outputs
                .as_deref()
                .and_then(|outputs| outputs.get(&source))
                .is_some_and(|link| link.bridges.iter().any(|bridge| bridge.output))
        {
            return Ok(());
        }
        let root = crate::sema::inline_constants::RootFacts {
            node: *self
                .facts
                .node(source)
                .ok_or_else(|| self.error(source.node, "missing completed inline root facts"))?,
            reference: self.facts.reference_adjustments.get(&source).copied(),
            pointer: self.facts.pointer_adjustments.get(&source).copied(),
            unsizing: self.facts.unsizing_adjustments.get(&source).copied(),
        };
        self.facts
            .inline_constants
            .as_mut()
            .ok_or_else(|| {
                crate::sema::model::Declarations::error(
                    self.report,
                    source,
                    "missing inline root registry",
                )
            })?
            .retain_root(source, root)
            .map_err(|e| self.error(source.node, e))
    }
    pub(super) fn publish_active_output(&mut self) -> Result<(), Error> {
        let Some(scheduler) = self.state.closures.take() else {
            return Ok(());
        };
        let result = match scheduler.active {
            Some(index) => {
                let parent = scheduler
                    .stack
                    .last()
                    .map(|(_, parent)| parent.body)
                    .ok_or_else(|| {
                        self.error(self.state.body.node, "missing output publication parent")
                    })?;
                self.publish_nested_output(scheduler.entries[index].payload.lifetimes(), parent)
            }
            None => Ok(()),
        };
        self.state.closures = Some(scheduler);
        result
    }
    pub(in crate::sema::bodies) fn prepare_nested_output(
        &mut self,
        interface: &mut Interface,
        parent: NodeKey,
    ) -> Result<(), Error> {
        let source = self.state.body;
        let result = self
            .state
            .result_term
            .ok_or_else(|| self.error(source.node, "missing nested result"))?;
        let mut observe = Bound::observe_regions();
        self.state
            .infer
            .map_regions(&mut self.facts.types, result, &mut observe)
            .map_err(|e| self.error(source.node, e))?;
        let imported: HashMap<_, _> = interface
            .opening
            .universal_pairs()
            .filter(|(caller, _)| {
                caller.bound_parts().is_none()
                    && caller.0 != 0
                    && self
                        .facts
                        .body_regions
                        .get(parent, *caller)
                        .is_some_and(|fact| fact.body == parent)
            })
            .map(|(caller, private)| (private, caller))
            .collect();
        let mut regions: Vec<_> = observe
            .observed_regions()
            .filter_map(|region| {
                if let Some(&caller) = imported.get(&region) {
                    return Some((region, Some(caller)));
                }
                (region.bound_parts().is_none()
                    && region.0 != 0
                    && self
                        .facts
                        .body_regions
                        .get(source, region)
                        .is_some_and(|fact| {
                            fact.body == source
                                && !matches!(fact.origin, BodyRegionOrigin::UniversalAlias)
                        }))
                .then_some((region, None))
            })
            .collect();
        regions.sort_unstable_by_key(|(region, _)| region.0);
        self.facts.body_work.canonical_steps += observe.steps + interface.opening.universal_count();
        for (private, caller) in regions {
            if interface
                .outputs
                .as_ref()
                .is_some_and(|ports| ports.pairs.contains_key(&private))
            {
                continue;
            }
            self.facts
                .nested_outputs
                .get_or_insert_with(Default::default)
                .reserve_port()
                .map_err(|e| self.error(source.node, e))?;
            let caller = if let Some(caller) = caller {
                caller
            } else {
                let caller =
                    self.body_region_for(source.node, parent, BodyRegionOrigin::Inference)?;
                interface
                    .opening
                    .region(caller, 0)
                    .map_err(|e| self.error(source.node, e))?;
                caller
            };
            let ports = interface.outputs.get_or_insert_with(|| {
                Box::new(Ports {
                    pairs: HashMap::default(),
                    parent,
                })
            });
            if ports.parent != parent || ports.pairs.len() >= 65_536 {
                return Err(self.error(
                    source.node,
                    "nested output interface ownership or size changed",
                ));
            }
            ports.pairs.insert(private, caller);
        }
        self.state.body_binders = interface.count().map_err(|e| self.error(source.node, e))?;
        self.prepare_universals(source.node)
    }

    pub(in crate::sema::bodies) fn publish_nested_output(
        &mut self,
        interface: &Interface,
        parent: NodeKey,
    ) -> Result<(), Error> {
        let source = self.state.body;
        let formation = if let Some(closure) = self.facts.closures.get(source)
            && let Some(environment) = self.facts.closures.environment(closure.identity)
        {
            let (inputs, work) = environment
                .formation_inputs()
                .map_err(|e| self.error(source.node, e))?;
            self.facts.body_work.canonical_steps += work;
            let mut queries = crate::types::outlives::Queries::new(
                &self.facts.types,
                &self.facts.lifetimes.nominal_requirements,
                &self.facts.lifetimes.callables,
            );
            let inputs: Vec<_> = inputs
                .into_iter()
                .filter(|input| {
                    !queries
                        .symbolic_requirements(input.child)
                        .is_ok_and(|requirements| {
                            requirements.scalars.regions.is_empty()
                                && requirements.scalars.parameters.is_empty()
                                && requirements.projections.is_empty()
                        })
                })
                .collect();
            (!inputs.is_empty()).then(|| Formation {
                identity: closure.identity,
                inputs: inputs.into_boxed_slice(),
            })
        } else {
            None
        };
        let captured = self.capture_input_ports(source, interface.outputs.is_none())?;
        if interface.outputs.is_none() && formation.is_none() && captured.is_empty() {
            return Ok(());
        }
        let parent_body = parent;
        let mut bridges = Vec::new();
        let outputs: HashSet<_> = interface
            .outputs
            .iter()
            .flat_map(|ports| ports.pairs.values())
            .copied()
            .collect();
        if outputs.is_empty() {
            // Closing maps visit both original inputs and their local aliases;
            // the bridge selector then visits the same incoming prefix once.
            self.facts.body_work.canonical_steps += 3 * interface.opening.universal_count();
        }
        // Output eligibility is narrower than ordinary input refresh.
        let mut closing = interface
            .closing(&self.state.call_types, true)
            .map_err(|e| self.error(source.node, e))?;
        for (parent, slot) in interface
            .opening
            .universal_pairs()
            .filter(|(_, slot)| !outputs.is_empty() || captured.contains(*slot))
        {
            // Public quantified inputs belong to this closure's callable binder,
            // even when the public result can mention them. They are not parent
            // proof identities and cannot discharge a captured output condition.
            if parent.bound_parts().is_some() || closing.region(slot, 0).is_err() {
                continue;
            }
            if outputs.is_empty() {
                self.validate_capture_input_parent(source, parent, parent_body)?;
            }
            let (_, slot) = slot
                .bound_parts()
                .ok_or_else(|| self.error(source.node, "missing output interface slot"))?;
            bridges.push(Bridge {
                slot,
                parent,
                output: outputs.contains(&parent),
            });
        }
        bridges.sort_unstable_by_key(|bridge| bridge.slot);
        let link = Link {
            source: source.into(),
            parent: parent.into(),
            revision: u32::try_from(outputs.len())
                .map_err(|_| self.error(source.node, "too many output ports"))?,
            universals: self.state.body_binders,
            bridges: bridges.into_boxed_slice(),
            formation,
        };
        self.facts
            .nested_outputs
            .get_or_insert_with(Default::default)
            .insert(source, link)
            .map_err(|e| self.error(source.node, e))
    }
}
