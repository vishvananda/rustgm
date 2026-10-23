//! Transactional original proofs retain no specialized execution authority.
use super::component_index::{Index, Node, Owner};
use super::{BodyKey, Contract, Declarations, Engine, Error, Report, Template, Templates, body};
use crate::pipeline::original_contexts::Original;
use crate::support::hash::{HashMap, HashSet};
const BYTES: usize = 64 * 1024 * 1024;
pub(super) struct Retained {
    index: Index,
    rows: HashMap<BodyKey, Original>,
    members: HashSet<BodyKey>,
    order: Vec<BodyKey>,
    pub(super) active: Option<Box<super::parent_scope::Active>>,
}
impl Engine {
    pub(super) fn component_original(&self, key: BodyKey) -> Option<&Original> {
        self.original_components.as_ref()?.rows.get(&key)
    }
    pub(super) fn component_contains(&self, key: BodyKey) -> bool {
        self.original_components
            .as_ref()
            .is_some_and(|c| c.members.contains(&key))
    }
    pub(super) fn original_component(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        seed: Original,
    ) -> Result<(Template, Contract), Error> {
        let origin = seed.template.body.source;
        let source = origin.source().ok_or_else(|| {
            Error::unlocated("source component cannot collect an imported original")
        })?;
        let mut work = 0;
        let operation = (|| {
            let mut retained = if let Some(retained) = self.original_components.take() {
                retained
            } else {
                let header = size_of::<Retained>().saturating_sub(size_of::<Index>())
                    + 2 * size_of::<usize>();
                let available = BYTES
                    .saturating_sub(facts.constant_work.template_bytes)
                    .saturating_sub(header);
                let index = Index::new(facts, available, &mut work)
                    .map_err(|e| report.body_error(source, e))?;
                facts.constant_work.template_bytes = facts
                    .constant_work
                    .template_bytes
                    .saturating_add(index.bytes)
                    .saturating_add(header);
                Box::new(Retained {
                    index,
                    rows: HashMap::default(),
                    members: HashSet::default(),
                    order: Vec::new(),
                    active: None,
                })
            };
            let result = self.complete_component(report, facts, &mut retained, seed, &mut work);
            self.original_components = Some(retained);
            result
        })();
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
        operation
    }
    fn complete_component(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        retained: &mut Retained,
        seed: Original,
        work: &mut usize,
    ) -> Result<(Template, Contract), Error> {
        let origin = seed.template.body.source;
        let source = origin.source().ok_or_else(|| {
            Error::unlocated("source component cannot collect an imported original")
        })?;
        let error = |e| report.body_error(source, e);
        let members = retained
            .index
            .component(facts, source, work)
            .map_err(error)?;
        let scratch = members
            .len()
            .saturating_mul(size_of::<Original>() + size_of::<Contract>())
            .saturating_add(crate::support::retained::new_map::<BodyKey, Original>(
                members.len(),
            ))
            .saturating_add(crate::support::retained::new_map::<BodyKey, ()>(
                members.len(),
            ));
        let mut bytes = facts.constant_work.template_bytes.saturating_add(scratch);
        if bytes > BYTES {
            return Err(error("original component scratch exceeds storage budget"));
        }
        let mut seed = Some(seed);
        let mut rows = Vec::with_capacity(members.len());
        for member in members {
            let node = retained.index.node(member).map_err(error)?;
            if retained.members.contains(&node.key()) {
                return Err(error("original component overlaps a completed component"));
            }
            let row = if node.source == source {
                seed.take()
                    .ok_or_else(|| error("duplicate component seed"))?
            } else {
                self.component_row(report, facts, node, BYTES.saturating_sub(bytes))?
            };
            row.validate(facts).map_err(error)?;
            bytes = bytes
                .saturating_add(row.template.retained_bytes())
                // The ordinary verifier temporarily retains its own Contract
                // payload. The original proof graph is not shared or dropped.
                .saturating_add(row.contract.retained_bytes().saturating_mul(2));
            if bytes > BYTES {
                return Err(error("original component bodies exceed storage budget"));
            }
            rows.push(row);
        }
        if seed.is_some() {
            return Err(error("component lost its requested original"));
        }
        self.prove_component(report, facts, rows, retained)?;
        self.anonymous
            .remove(&source)
            .ok_or_else(|| error("component lost its proved anonymous original"))
    }
    fn component_row(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        node: &Node,
        available: usize,
    ) -> Result<Original, Error> {
        let source = node.source;
        let error = |e| report.body_error(source, e);
        let (mut template, contract) = match node.owner {
            Owner::Function(id) => {
                let function = facts
                    .function_indices
                    .get(&id)
                    .and_then(|&i| facts.functions.get(i))
                    .copied()
                    .ok_or_else(|| error("missing component function"))?;
                let input = self.contracts.function(facts, &function).map_err(error)?;
                let clone_bytes = input
                    .retained_bytes()
                    .saturating_add(self.originals.get(&id).map_or(0, Template::retained_bytes));
                if clone_bytes > available {
                    return Err(error(
                        "original function copy exceeds component storage budget",
                    ));
                }
                let contract = input.clone();
                let template = if let Some(t) = self.originals.get(&id) {
                    t.clone()
                } else {
                    self.construction.function(report, facts, &function)?
                };
                (template, contract)
            }
            Owner::Closure(id) => {
                let closure = facts
                    .closures
                    .identity(id)
                    .copied()
                    .ok_or_else(|| error("missing component closure"))?;
                let owner = closure
                    .owner
                    .ok_or_else(|| error("component closure has no declaration"))?;
                (
                    self.construction.closure(report, facts, &closure, owner)?,
                    self.contracts.closure(facts, &closure).map_err(error)?,
                )
            }
            Owner::Initializer(id) => {
                if let Some((template, contract)) =
                    self.associated.as_deref().and_then(|rows| rows.get(&id))
                {
                    return copy_original(template, contract, available).map_err(error);
                }
                let initializer = facts
                    .initializer_index(id)
                    .and_then(|i| facts.initializers.get(i))
                    .copied()
                    .ok_or_else(|| error("missing component initializer"))?;
                let mut template = self.construction.initializer(report, facts, &initializer)?;
                let mut contract = self
                    .contracts
                    .initializer(facts, &initializer)
                    .map_err(error)?;
                body::dispatch::original_initializer(
                    facts,
                    &mut template.body,
                    &mut contract,
                    &initializer,
                )
                .map_err(|e| report.body_error(source, e))?;
                (template, contract)
            }
            Owner::Anonymous => {
                let (template, contract) = if let Some(row) = self.anonymous.get(&source) {
                    if row
                        .0
                        .retained_bytes()
                        .saturating_add(row.1.retained_bytes())
                        > available
                    {
                        return Err(error("anonymous copy exceeds component storage budget"));
                    }
                    row.clone()
                } else {
                    self.build_anonymous(report, facts, source)?
                };
                return Ok(Original { template, contract });
            }
        };
        body::promote::instance_reusing(
            report,
            facts,
            &mut template.body,
            &template.destruction,
            &mut self.promotions,
        )?;
        Ok(Original { template, contract })
    }
    fn prove_component(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        rows: Vec<Original>,
        retained: &mut Retained,
    ) -> Result<(), Error> {
        let mut templates = Templates::source(Vec::with_capacity(rows.len()));
        let mut contracts = Vec::with_capacity(rows.len());
        for mut row in rows {
            crate::body::copy_requirements::prepare(facts, &row.template.body, &mut row.contract)
                .map_err(|e| report.body_error(row.contract.source, e))?;
            row.contract
                .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
                .map_err(|e| report.body_error(row.contract.source, e))?;
            templates.bodies.push(row.template);
            contracts.push(row.contract);
        }
        let contracts = crate::body::regions::Contracts {
            bodies: contracts,
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        };
        let custom = facts
            .destructors
            .implementations()
            .iter()
            .map(|i| i.item)
            .collect::<Vec<_>>();
        let mut copies = crate::types::copy::Copies::default();
        let checked = crate::borrow::templates::verify(
            &mut crate::borrow::Environment {
                types: &facts.types,
                definitions: &facts.aggregates,
                derives: &facts.derives,
                source_copy: facts.uses_source_copy(),
                copies: &mut copies,
            },
            &facts.lifetimes.variances,
            &contracts,
            &templates,
            &custom,
            |ty, field| body::drops::nominal_position(facts, ty, field),
        )
        .map_err(|e| {
            report.body_error(
                e.node.unwrap_or(templates.bodies[0].body.source.into()),
                e.message,
            )
        })?;
        let inputs = contracts.bodies;
        super::defining_heads::close_component(report, facts, &templates, &checked)?;
        drop(checked);
        self.publish_component(facts, templates, inputs, retained)
    }
    fn publish_component(
        &mut self,
        facts: &mut Declarations,
        templates: Templates,
        contracts: Vec<Contract>,
        retained: &mut Retained,
    ) -> Result<(), Error> {
        // All fallible proof and storage work precedes publication of any row.
        let count = retained
            .members
            .len()
            .saturating_add(templates.bodies.len());
        let bytes = templates
            .bodies
            .iter()
            .map(Template::retained_bytes)
            .chain(contracts.iter().map(Contract::retained_bytes))
            .fold(0_usize, usize::saturating_add)
            .saturating_add(
                crate::support::retained::new_map::<BodyKey, Original>(count)
                    .saturating_add(crate::support::retained::new_map::<BodyKey, ()>(count))
                    .saturating_add(count.saturating_mul(2 * size_of::<BodyKey>()))
                    .saturating_sub(
                        crate::support::retained::new_map::<BodyKey, Original>(
                            retained.members.len(),
                        )
                        .saturating_add(crate::support::retained::new_map::<BodyKey, ()>(
                            retained.members.len(),
                        ))
                        .saturating_add(
                            retained
                                .members
                                .len()
                                .saturating_mul(2 * size_of::<BodyKey>()),
                        ),
                    ),
            );
        let total = facts.constant_work.template_bytes.saturating_add(bytes);
        if total > BYTES {
            return Err(Error::at(
                super::FileId(0),
                0,
                "original component retention exceeds storage budget",
            ));
        }
        facts.constant_work.template_bytes = total;
        for (template, contract) in templates.bodies.into_iter().zip(contracts) {
            let key = template.body.key();
            if matches!(template.body.owner, body::ir::Owner::AnonymousConstant) {
                let source = template.body.source.source().ok_or_else(|| {
                    Error::unlocated("source component cannot publish imported anonymous syntax")
                })?;
                facts.constant_work.anonymous_templates +=
                    usize::from(!self.anonymous.contains_key(&source));
                self.anonymous.insert(source, (template, contract));
            } else {
                retained.rows.insert(key, Original { template, contract });
            }
            retained.members.insert(key);
            retained.order.push(key);
        }
        Ok(())
    }
    pub(super) fn transfer_components(
        &self,
        report: &Report,
        facts: &mut Declarations,
        prepared: &mut crate::pipeline::Prepared,
    ) -> Result<(), Error> {
        let Some(retained) = &self.original_components else {
            return Ok(());
        };
        let mut bytes = retained.members.len().saturating_mul(size_of::<Original>());
        for &key in &retained.order {
            let (template, contract) = if let BodyKey::Anonymous(source) = key {
                let row = self.anonymous.get(&source).ok_or_else(|| {
                    report.body_error(source, "component anonymous cache was lost")
                })?;
                (&row.0, &row.1)
            } else {
                let row = retained.rows.get(&key).ok_or_else(|| {
                    Error::at(super::FileId(0), 0, "component parent cache was lost")
                })?;
                (&row.template, &row.contract)
            };
            bytes = bytes
                .saturating_add(template.retained_bytes())
                .saturating_add(contract.retained_bytes());
        }
        if facts.constant_work.template_bytes.saturating_add(bytes) > BYTES {
            return Err(Error::at(
                super::FileId(0),
                0,
                "original component handoff copy exceeds storage budget",
            ));
        }
        let mut rows = Vec::with_capacity(retained.members.len());
        for &key in &retained.order {
            let (template, contract) = if let BodyKey::Anonymous(source) = key {
                let row = self.anonymous.get(&source).ok_or_else(|| {
                    report.body_error(source, "component anonymous cache was lost")
                })?;
                (&row.0, &row.1)
            } else {
                let row = retained.rows.get(&key).ok_or_else(|| {
                    Error::at(super::FileId(0), 0, "component parent cache was lost")
                })?;
                (&row.template, &row.contract)
            };
            rows.push(Original {
                template: template.clone(),
                contract: contract.clone(),
            });
        }
        prepared
            .original_contexts
            .get_or_insert_with(Default::default)
            .publish(facts, &mut prepared.templates, rows)
            .map_err(|e| Error::at(super::FileId(0), 0, e))
    }
}

fn copy_original(
    template: &Template,
    contract: &Contract,
    available: usize,
) -> Result<Original, &'static str> {
    if template
        .retained_bytes()
        .saturating_add(contract.retained_bytes())
        > available
    {
        return Err("initializer copy exceeds component storage budget");
    }
    Ok(Original {
        template: template.clone(),
        contract: contract.clone(),
    })
}

#[cfg(test)]
#[path = "../../../tests/invariants/parent_reuse.rs"]
mod tests;
