//! Orchestrate append-only body proofs without retaining semantic borrows across demand.
#[cfg(test)]
mod tests;
use crate::{
    body::{
        drops,
        ir::{Body, Program},
        regions::Contract,
    },
    borrow::{
        self,
        destruction::{dependencies::Dependencies, glue},
        incremental, templates,
    },
    sema::{self, model::Declarations},
    types::{instance::Instance, outlives::QueryCache},
};

pub(super) struct Builder {
    program: Program,
    bodies: Vec<incremental::Proved>,
    queries: QueryCache,
    verifier: incremental::Verifier,
    recipes: Option<glue::Cache>,
    dependencies: Dependencies,
    adapters: bool,
    original_regions: Option<crate::support::hash::HashMap<crate::body::ir::BodyKey, RegionProof>>,
}
enum RegionProof {
    Ordinary(borrow::regions::Body),
    Parent(borrow::regions::Retained),
}
pub(super) enum Staged {
    Ordinary(Box<incremental::PairedProved>),
    Parent(incremental::OwnedParent),
}
impl Staged {
    pub(super) fn body(&self) -> &Body {
        match self {
            Self::Ordinary(p) => p.body(),
            Self::Parent(p) => p.body(),
        }
    }
    pub(super) fn paired(&self) -> &incremental::PairedProved {
        match self {
            Self::Ordinary(p) => p,
            Self::Parent(p) => p.paired(),
        }
    }
}
enum Original<'a> {
    Ordinary,
    Factory(crate::body::primitive_methods::Permit),
    Adapter(usize, &'a crate::body::adapters::Template),
}
impl Builder {
    pub(super) fn new(mut program: Program) -> Result<(Self, Vec<Body>), &'static str> {
        let ordinary = std::mem::take(&mut program.bodies);
        // Whole-program region tables exist only for body inspection views;
        // proofs and lowering read the per-body contracts instead.
        program.region_equalities = Vec::new();
        program.coercions = Vec::new();
        program.outlives = Vec::new();
        program.body_regions = Vec::new();
        let methods = program.drops.as_ref().map(|d| d.methods.as_slice());
        let recipes = methods.map(glue::Cache::new).transpose()?;
        let verifier = incremental::Verifier::new(methods)?;
        Ok((
            Self {
                program,
                bodies: Vec::with_capacity(ordinary.len()),
                queries: QueryCache::default(),
                verifier,
                recipes,
                dependencies: Dependencies::default(),
                adapters: false,
                original_regions: None,
            },
            ordinary,
        ))
    }
    pub(super) fn body(&self, index: usize) -> Result<&Body, &'static str> {
        self.bodies
            .get(index)
            .map(incremental::Proved::body)
            .ok_or("missing proved instance body")
    }
    pub(super) const fn has_adapters(&self) -> bool {
        self.adapters
    }
    pub(super) fn prepare_original_regions(
        &mut self,
        facts: &mut Declarations,
        inputs: &crate::support::hash::HashMap<crate::body::ir::BodyKey, Contract>,
        bodies: &[Body],
    ) -> Result<(), borrow::Error> {
        if !bodies.iter().any(|body| {
            inputs
                .get(&body.key())
                .is_some_and(|input| input.output.is_some())
        }) {
            return Ok(());
        }
        let sources: crate::support::hash::HashSet<_> = bodies
            .iter()
            .filter_map(|body| {
                inputs
                    .get(&body.key())
                    .and_then(|input| input.output.as_ref())
                    .map(|link| [body.source, link.parent])
            })
            .flatten()
            .collect();
        let bodies: Vec<_> = bodies
            .iter()
            .filter(|body| sources.contains(&body.source))
            .collect();
        for body in &bodies {
            let input = inputs.get(&body.key()).ok_or_else(|| borrow::Error {
                node: Some(body.source.into()),
                message: "missing original output contract".into(),
            })?;
            input
                .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
                .map_err(|message| borrow::Error {
                    node: Some(body.source.into()),
                    message: message.into(),
                })?;
        }
        let contracts = crate::body::regions::Contracts {
            bodies: Vec::new(),
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        };
        let regions = borrow::regions::check_nested_bodies(
            &facts.types,
            &facts.lifetimes.variances,
            &contracts,
            bodies.iter().map(|&body| {
                inputs
                    .get(&body.key())
                    .map(|input| (input, body))
                    .ok_or_else(|| borrow::regions::Error {
                        source: Some(body.source.into()),
                        message: "missing original output contract".into(),
                    })
            }),
        )
        .map_err(|e| borrow::Error {
            node: e.source,
            message: e.message,
        })?;
        let mut retained = crate::support::hash::HashMap::with_capacity_and_hasher(
            regions.bodies.len(),
            crate::support::hash::BuildFxHasher::default(),
        );
        for proof in regions.bodies {
            if retained
                .insert(proof.key, RegionProof::Ordinary(proof))
                .is_some()
            {
                return Err(borrow::Error {
                    node: None,
                    message: "duplicate original region proof".into(),
                });
            }
        }
        self.original_regions = Some(retained);
        Ok(())
    }
    pub(super) fn prepare_parent_regions(
        &mut self,
        facts: &mut Declarations,
        inputs: &crate::support::hash::HashMap<crate::body::ir::BodyKey, std::sync::Arc<Contract>>,
        bodies: &[Body],
    ) -> Result<(), borrow::Error> {
        let error = |message: &str| borrow::Error {
            node: None,
            message: message.into(),
        };
        for body in bodies {
            inputs
                .get(&body.key())
                .ok_or_else(|| error("conditional group lost its contract"))?
                .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
                .map_err(error)?;
        }
        let contracts = crate::body::regions::Contracts {
            bodies: Vec::new(),
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        };
        let pairs = bodies
            .iter()
            .map(|body| {
                inputs
                    .get(&body.key())
                    .map(|input| (std::sync::Arc::clone(input), body))
                    .ok_or_else(|| error("conditional group lost its contract"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let retained = borrow::regions::check_retained_group(
            &facts.types,
            &facts.lifetimes.variances,
            &contracts,
            pairs,
        )
        .map_err(|e| borrow::Error {
            node: e.source,
            message: e.message,
        })?;
        self.original_regions = Some(
            retained
                .into_iter()
                .map(|proof| (proof.input().key, RegionProof::Parent(proof)))
                .collect(),
        );
        Ok(())
    }
    pub(super) fn stage_parent(
        &mut self,
        facts: &mut Declarations,
        templates: &templates::Checked<'_>,
        body: Body,
        pending: Option<crate::body::conditional_inputs::Pending>,
    ) -> Result<Staged, borrow::Error> {
        let source = body.source;
        let retained = self
            .original_regions
            .as_mut()
            .and_then(|r| r.remove(&body.key()))
            .ok_or_else(|| borrow::Error {
                node: Some(source.into()),
                message: "missing paired conditional region proof".into(),
            })?;
        let RegionProof::Parent(retained) = retained else {
            return Err(borrow::Error {
                node: Some(source.into()),
                message: "conditional parent has an ordinary region cache row".into(),
            });
        };
        let mut catalog = self
            .recipes
            .take()
            .map(|cache| cache.attach(&facts.types, &facts.aggregates, &facts.members));
        let mut environment = borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            source_copy: facts.uses_source_copy(),
            copies: &mut facts.copies,
        };
        let proved = match pending {
            Some(pending) => self
                .verifier
                .prove_parent(
                    &mut environment,
                    templates,
                    retained,
                    body,
                    (pending, &mut facts.body_work.canonical_steps),
                    catalog.as_mut(),
                )
                .map(Staged::Parent),
            None => self
                .verifier
                .prove_paired(
                    &mut environment,
                    templates,
                    retained,
                    body,
                    catalog.as_mut(),
                )
                .map(|proved| Staged::Ordinary(Box::new(proved))),
        };
        self.recipes = catalog.map(glue::Catalog::into_cache);
        proved
    }
    pub(super) fn append_proved(
        &mut self,
        facts: &mut Declarations,
        mut proof: incremental::Proved,
    ) -> Result<usize, borrow::Error> {
        proof
            .resolve_callables(&mut facts.types)
            .map_err(|message| borrow::Error {
                node: Some(proof.body().source.into()),
                message: message.into(),
            })?;
        let index = self.bodies.len();
        self.adapters |= proof
            .body()
            .closure_calls
            .as_deref()
            .is_some_and(crate::body::invocation_entries::Entries::has_adapters);
        self.bodies.push(proof);
        Ok(index)
    }
    pub(super) fn push(
        &mut self,
        facts: &mut Declarations,
        templates: &templates::Checked<'_>,
        body: Body,
        input: &Contract,
        requirements: Option<drops::Requirements>,
    ) -> Result<usize, borrow::Error> {
        self.push_inner(
            facts,
            templates,
            body,
            input,
            requirements,
            Original::Ordinary,
        )
    }
    pub(super) fn push_factory(
        &mut self,
        facts: &mut Declarations,
        templates: &templates::Checked<'_>,
        selected: crate::sema::primitive_methods::Selected,
        body: Body,
        input: &Contract,
    ) -> Result<usize, borrow::Error> {
        let permit = crate::body::primitive_methods::validate(facts, selected, &body, input)
            .map_err(|message| borrow::Error {
                node: Some(body.source.into()),
                message: message.into(),
            })?;
        let mut query = crate::types::destruction::NeedsDrop::new(
            facts
                .destructors
                .implementations()
                .iter()
                .map(|implementation| implementation.item),
        );
        let requirements =
            crate::body::drops::requirements(facts, &body, &mut query).map_err(|message| {
                borrow::Error {
                    node: Some(body.source.into()),
                    message: message.into(),
                }
            })?;
        self.push_inner(
            facts,
            templates,
            body,
            input,
            Some(requirements),
            Original::Factory(permit),
        )
    }
    pub(super) fn push_adapter(
        &mut self,
        report: &crate::resolve::expansion::Report,
        facts: &mut Declarations,
        templates: &templates::Checked<'_>,
        source: usize,
        mut prepared: crate::body::adapters::Prepared,
    ) -> Result<usize, borrow::Error> {
        let error = |message: &str| borrow::Error {
            node: Some(prepared.body.source.into()),
            message: message.into(),
        };
        prepared
            .template
            .validate(
                &facts.types,
                self.body(source).map_err(error)?,
                &prepared.body,
            )
            .map_err(error)?;
        prepared
            .template
            .validate_contract(&facts.types, &prepared.input)
            .map_err(error)?;
        let mut query = crate::types::destruction::NeedsDrop::new(
            facts
                .destructors
                .implementations()
                .iter()
                .map(|implementation| implementation.item),
        );
        let requirements = crate::body::drops::requirements(facts, &prepared.body, &mut query)
            .map_err(|message| borrow::Error {
                node: Some(prepared.body.source.into()),
                message: message.into(),
            })?;
        crate::body::promote::instance(report, facts, &mut prepared.body, &requirements).map_err(
            |error| borrow::Error {
                node: Some(prepared.body.source.into()),
                message: error.to_string(),
            },
        )?;
        self.push_inner(
            facts,
            templates,
            prepared.body,
            &prepared.input,
            Some(requirements),
            Original::Adapter(source, &prepared.template),
        )
    }
    fn push_inner(
        &mut self,
        facts: &mut Declarations,
        templates: &templates::Checked<'_>,
        body: Body,
        input: &Contract,
        requirements: Option<drops::Requirements>,
        original: Original<'_>,
    ) -> Result<usize, borrow::Error> {
        let contract = self.regions(facts, &body, input)?;
        let mut catalog = self
            .recipes
            .take()
            .map(|cache| cache.attach(&facts.types, &facts.aggregates, &facts.members));
        let mut environment = borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            source_copy: facts.uses_source_copy(),
            copies: &mut facts.copies,
        };
        let proof = match original {
            Original::Adapter(index, template) => {
                let source = self.bodies.get(index).ok_or_else(|| borrow::Error {
                    node: Some(input.source.into()),
                    message: "receiver adapter has no proved original source".into(),
                })?;
                self.verifier.prove_adapter(
                    &mut environment,
                    templates,
                    &contract,
                    body,
                    catalog.as_mut(),
                    (source, template),
                )
            }
            ordinary => {
                let factory = if let Original::Factory(permit) = ordinary {
                    Some(permit)
                } else {
                    None
                };
                self.verifier.prove(
                    &mut environment,
                    templates,
                    &contract,
                    body,
                    catalog.as_mut(),
                    factory,
                )
            }
        };
        self.recipes = catalog.map(glue::Catalog::into_cache);
        let mut proof = proof?;
        proof
            .resolve_callables(&mut facts.types)
            .map_err(|message| borrow::Error {
                node: Some(input.source.into()),
                message: message.into(),
            })?;
        let index = self.bodies.len();
        self.adapters |= proof
            .body()
            .closure_calls
            .as_deref()
            .is_some_and(crate::body::invocation_entries::Entries::has_adapters);
        self.bodies.push(proof);
        if let (Some(pending), Some(requirements)) = (&mut self.program.drops, requirements) {
            pending.bodies.push(requirements);
        }
        Ok(index)
    }
    fn regions(
        &mut self,
        facts: &mut Declarations,
        body: &Body,
        input: &Contract,
    ) -> Result<borrow::regions::Body, borrow::Error> {
        input
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(|message| borrow::Error {
                node: Some(input.source.into()),
                message: message.into(),
            })?;
        let mut queries = std::mem::take(&mut self.queries).attach(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let result = if body.instance.is_none() {
            self.original_regions
                .as_mut()
                .and_then(|regions| regions.remove(&body.key()))
                .map(|proof| match proof {
                    RegionProof::Ordinary(proof) => Ok(proof),
                    RegionProof::Parent(_) => Err(borrow::regions::Error {
                        source: Some(body.source.into()),
                        message: "paired parent proof requires its actual owner path".into(),
                    }),
                })
        } else {
            None
        }
        .unwrap_or_else(|| {
            borrow::regions::check_one(
                &facts.types,
                &facts.lifetimes.variances,
                &mut queries,
                input,
                body,
            )
        });
        self.queries = queries.into_cache();
        result.map_err(|e| borrow::Error {
            node: e.source,
            message: e.message,
        })
    }
    pub(super) fn cleanup_dependencies(
        &mut self,
        facts: &mut Declarations,
        index: usize,
    ) -> Result<Vec<Instance>, &'static str> {
        let Some(plan) = self.bodies.get(index).and_then(incremental::Proved::plan) else {
            return Ok(Vec::new());
        };
        let cache = self
            .recipes
            .as_mut()
            .ok_or("missing instance cleanup recipes")?;
        let requests = self.dependencies.collect(cache.recipes(), plan)?;
        self.select_cleanup(facts, &requests)
    }
    pub(super) fn object_dependencies(
        &mut self,
        facts: &mut Declarations,
        types: &[crate::types::TypeId],
    ) -> Result<Vec<Instance>, &'static str> {
        // No destructor declarations means every concrete sized type is dropless.
        let Some(cache) = self.recipes.take() else {
            return Ok(Vec::new());
        };
        let mut catalog = cache.attach(&facts.types, &facts.aggregates, &facts.members);
        let roots = types
            .iter()
            .map(|&ty| catalog.expose(ty))
            .collect::<Result<Vec<_>, _>>();
        self.recipes = Some(catalog.into_cache());
        let cache = self
            .recipes
            .as_ref()
            .ok_or("missing object destruction recipes")?;
        let requests = self
            .dependencies
            .exposed(cache.recipes(), roots?.into_iter().flatten())?;
        self.select_cleanup(facts, &requests)
    }
    fn select_cleanup(
        &mut self,
        facts: &mut Declarations,
        requests: &[glue::Id],
    ) -> Result<Vec<Instance>, &'static str> {
        let cache = self
            .recipes
            .as_mut()
            .ok_or("missing selected destruction recipes")?;
        let mut methods = Vec::with_capacity(requests.len());
        for &id in requests {
            let recipe = *cache
                .recipes()
                .get(id)
                .ok_or("missing demanded cleanup recipe")?;
            let instance = match recipe.method {
                Some(glue::Method::Definition(method)) => {
                    let ty = facts
                        .definitions
                        .get(method.0 as usize)
                        .copied()
                        .flatten()
                        .ok_or("missing destructor function item")?;
                    Instance::new(&facts.types, ty)?
                }
                Some(glue::Method::Unresolved(_)) => {
                    let instance = sema::destructors::select(facts, recipe.ty)?
                        .ok_or("missing concrete destructor selection")?;
                    cache.resolve(&mut facts.types, id, instance)?;
                    instance
                }
                Some(glue::Method::Instance(instance)) => instance,
                None => return Err("cleanup dependency has no method"),
            };
            methods.push(instance);
        }
        Ok(methods)
    }
    #[cfg(test)]
    pub(super) fn with_checked<T>(
        self,
        use_checked: impl FnOnce(&borrow::Checked<'_>) -> T,
    ) -> Result<T, borrow::Error> {
        Ok(self.assemble()?.with_checked(use_checked))
    }
    /// Complete the append-only proof; every original nested region proof
    /// must have been consumed by its body.
    pub(super) fn assemble(self) -> Result<incremental::Assembled, borrow::Error> {
        if self
            .original_regions
            .as_ref()
            .is_some_and(|proofs| !proofs.is_empty())
        {
            return Err(borrow::Error {
                node: None,
                message: "unconsumed original nested region proof".into(),
            });
        }
        incremental::assemble(
            self.program,
            self.bodies,
            self.recipes.map(glue::Cache::into_recipes),
        )
    }
}
