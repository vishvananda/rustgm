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
    original_regions:
        Option<std::collections::HashMap<crate::body::ir::BodyKey, borrow::regions::Body>>,
}
impl Builder {
    pub(super) fn new(mut program: Program) -> Result<(Self, Vec<Body>), &'static str> {
        let ordinary = std::mem::take(&mut program.bodies);
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
    pub(super) fn prepare_original_regions(
        &mut self,
        facts: &mut Declarations,
        inputs: &std::collections::HashMap<crate::body::ir::BodyKey, Contract>,
        bodies: &[Body],
    ) -> Result<(), borrow::Error> {
        if !bodies.iter().any(|body| {
            inputs
                .get(&body.key())
                .is_some_and(|input| input.output.is_some())
        }) {
            return Ok(());
        }
        let sources: std::collections::HashSet<_> = bodies
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
                node: Some(body.source),
                message: "missing original output contract".into(),
            })?;
            input
                .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
                .map_err(|message| borrow::Error {
                    node: Some(body.source),
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
                        source: Some(body.source),
                        message: "missing original output contract".into(),
                    })
            }),
        )
        .map_err(|e| borrow::Error {
            node: e.source,
            message: e.message,
        })?;
        let mut retained = std::collections::HashMap::with_capacity(regions.bodies.len());
        for proof in regions.bodies {
            if retained.insert(proof.key, proof).is_some() {
                return Err(borrow::Error {
                    node: None,
                    message: "duplicate original region proof".into(),
                });
            }
        }
        self.original_regions = Some(retained);
        Ok(())
    }
    pub(super) fn push(
        &mut self,
        facts: &mut Declarations,
        templates: &templates::Checked<'_>,
        body: Body,
        input: &Contract,
        requirements: Option<drops::Requirements>,
    ) -> Result<usize, borrow::Error> {
        self.push_inner(facts, templates, body, input, requirements, None)
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
                node: Some(body.source),
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
                    node: Some(body.source),
                    message: message.into(),
                }
            })?;
        self.push_inner(
            facts,
            templates,
            body,
            input,
            Some(requirements),
            Some(permit),
        )
    }
    fn push_inner(
        &mut self,
        facts: &mut Declarations,
        templates: &templates::Checked<'_>,
        body: Body,
        input: &Contract,
        requirements: Option<drops::Requirements>,
        factory: Option<crate::body::primitive_methods::Permit>,
    ) -> Result<usize, borrow::Error> {
        input
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(|message| borrow::Error {
                node: Some(input.source),
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
                .map(Ok)
        } else {
            None
        }
        .unwrap_or_else(|| {
            borrow::regions::check_one(
                &facts.types,
                &facts.lifetimes.variances,
                &mut queries,
                input,
                &body,
            )
        });
        self.queries = queries.into_cache();
        let contract = result.map_err(|e| borrow::Error {
            node: e.source,
            message: e.message,
        })?;
        let mut catalog = self
            .recipes
            .take()
            .map(|cache| cache.attach(&facts.types, &facts.aggregates, &facts.members));
        let mut environment = borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            copies: &mut facts.copies,
        };
        let proof = self.verifier.prove(
            &mut environment,
            templates,
            &contract,
            body,
            catalog.as_mut(),
            factory,
        );
        self.recipes = catalog.map(glue::Catalog::into_cache);
        let mut proof = proof?;
        proof
            .resolve_callables(&mut facts.types)
            .map_err(|message| borrow::Error {
                node: Some(input.source),
                message: message.into(),
            })?;
        let index = self.bodies.len();
        self.bodies.push(proof);
        if let (Some(pending), Some(requirements)) = (&mut self.program.drops, requirements) {
            pending.bodies.push(requirements);
        }
        Ok(index)
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
    pub(super) fn with_checked<T>(
        self,
        use_checked: impl FnOnce(&borrow::Checked<'_>) -> T,
    ) -> Result<T, borrow::Error> {
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
        incremental::with_checked(
            self.program,
            self.bodies,
            self.recipes.map(glue::Cache::into_recipes),
            use_checked,
        )
    }
}
