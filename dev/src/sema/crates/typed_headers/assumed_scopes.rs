//! Private declaration assumptions share canonical clauses and Environment indexes.
use super::{Prepared, Progress, Ready, Session, State, formal_path};
use crate::{
    resolve::{
        crates::{ReservationWork, unpaid_provided::RecipeView},
        package::Error,
    },
    sema::{
        crates::typed_headers::{
            error,
            predicate_remap::translation,
            recipe_cursor::{Budget, Demand, Dependency, Port, Reserved},
            staged_headers::environments::{self, Accounting},
        },
        declaration_inputs::Input,
        model::Declarations,
    },
    support::{
        crates::{generics::source as s, owner_catalog::DeclarationRef},
        retained::{self, Allocation},
    },
    types::{
        arguments::Argument,
        parameters::{
            ParameterId,
            clauses::Clause,
            environment::{Abilities, Environment},
            predicates::Predicate,
        },
    },
};
use std::sync::Arc;
#[path = "assumed_originals.rs"]
mod originals;
#[path = "assumption_ports.rs"]
mod ports;
#[cfg(test)]
#[path = "../../../../tests/invariants/assumed_environments.rs"]
mod tests;

struct Scope {
    raw: s::ParameterScope,
    inputs: Arc<[Input]>,
    subjects: Box<[bool]>,
    parameters: Vec<(ParameterId, Abilities)>,
    clauses: Vec<Clause>,
    environment: Arc<Environment>,
}
impl Scope {
    fn prepare(
        port: &mut ports::Inputs<'_, '_>,
        facts: &mut Declarations,
        scope_row: s::ParameterScope,
        inherited: Option<&Scope>,
        applications: &mut [Option<Predicate>],
        work: &mut ReservationWork,
    ) -> Result<(Self, usize), Error> {
        let rows = port.port.rows();
        let (range, canonical) = originals::input_range(rows, scope_row)?;
        let own = range.slice(&rows.clauses).map_err(error)?.len();
        let previous_parameters = inherited.map_or(0, |scope| scope.parameters.len());
        let previous_clauses = inherited.map_or(0, |scope| scope.clauses.len());
        let slots = scope_row.declared.slice(&rows.slots).map_err(error)?.len();
        let before = work.retained;
        let store_bytes = facts.types.trait_input_bytes();
        work.charge(
            previous_parameters
                .saturating_add(previous_clauses)
                .saturating_add(own),
        )?;
        work.reserve(
            size_of::<Scope>()
                .saturating_add(
                    (previous_parameters + slots)
                        .saturating_mul(size_of::<(ParameterId, Abilities)>()),
                )
                .saturating_add((previous_clauses + own).saturating_mul(size_of::<Clause>()))
                .saturating_add(2 * size_of::<usize>()),
        )?;
        let mut parameters = Vec::with_capacity(previous_parameters + slots);
        let mut clauses = Vec::with_capacity(previous_clauses + own);
        if let Some(parent) = inherited {
            parameters.extend_from_slice(&parent.parameters);
            clauses.extend_from_slice(&parent.clauses);
        }
        let definition = port.port.definition(scope_row.owner, work)?;
        formal_path::visit(
            &mut port.port,
            &mut facts.types,
            &scope_row,
            definition,
            work,
            |parameter| parameters.push((parameter, Abilities::default())),
        )?;
        let (inputs, subjects) = translation::inputs(
            port,
            applications,
            scope_row.owner,
            range,
            canonical,
            &mut facts.types,
            work,
        )?;
        clauses.extend(inputs.iter().map(|input| input.value));
        let mut accounting = Observed {
            work,
            allocation: Allocation::default(),
        };
        let environment = environments::prepare(
            facts,
            inherited.map(|scope| &scope.environment),
            &parameters,
            &clauses,
            previous_parameters,
            previous_clauses,
            &mut accounting,
        )?;
        let scope = Scope {
            raw: scope_row,
            inputs,
            subjects,
            parameters,
            clauses,
            environment,
        };
        let bytes = scope.payload_bytes(accounting.allocation.bytes);
        let actual =
            bytes.saturating_add(facts.types.trait_input_bytes().saturating_sub(store_bytes));
        if actual > accounting.work.retained.saturating_sub(before) {
            return Err(error(
                "private environment exceeded its selected allocation preflight",
            ));
        }
        accounting.work.retained = before.saturating_add(actual);
        Ok((scope, bytes))
    }
    fn payload_bytes(&self, environment: usize) -> usize {
        size_of::<Self>()
            .saturating_add(retained::vector(&self.parameters))
            .saturating_add(retained::vector(&self.clauses))
            .saturating_add(self.inputs.len().saturating_mul(size_of::<Input>()))
            .saturating_add(self.subjects.len())
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(environment)
    }
    fn require_empty(&self, expected: Ready) -> Result<(), Error> {
        if self.raw != expected.scope.1
            || self.raw.parent.is_some()
            || !self.parameters.is_empty()
            || !self.clauses.is_empty()
            || !self.inputs.is_empty()
            || !self.subjects.is_empty()
        {
            return Err(error(
                "empty Contract changed its complete prepared declaration inputs",
            ));
        }
        Ok(())
    }
    fn require(&self, rows: s::ValidatedView<'_>) -> Result<(), Error> {
        let (range, canonical) = originals::input_range(rows, self.raw)?;
        if self.inputs.len() != range.slice(&rows.clauses).map_err(error)?.len()
            || self.subjects.len() != canonical.unwrap_or(0)
        {
            return Err(error(
                "private declaration environment changed its complete canonical input owner",
            ));
        }
        Ok(())
    }
}
struct ContractBody {
    seal: originals::Origin,
    declaration: Arc<Environment>,
    environment: Arc<Environment>,
}
enum Body {
    Contract {
        expected: Ready,
        scope: usize,
        product: Box<ContractBody>,
    },
    Ready {
        expected: Ready,
        scope: usize,
        environment: Arc<Environment>,
    },
    Blocked {
        expected: Ready,
        dependency: Dependency,
    },
}
pub(super) struct Owner {
    scopes: Vec<Option<Box<Scope>>>,
    bodies: Vec<Option<Box<Body>>>,
    applications: Vec<Option<Predicate>>,
    bytes: usize,
}
impl Owner {
    fn new(rows: s::ValidatedView<'_>, work: &mut ReservationWork) -> Result<Self, Error> {
        work.charge(
            rows.scopes
                .len()
                .saturating_add(rows.functions.len())
                .saturating_add(rows.trait_applications.len()),
        )?;
        let bytes = size_of::<Self>()
            .saturating_add(
                rows.scopes
                    .len()
                    .saturating_mul(size_of::<Option<Box<Scope>>>()),
            )
            .saturating_add(
                rows.functions
                    .len()
                    .saturating_mul(size_of::<Option<Box<Body>>>()),
            )
            .saturating_add(
                rows.trait_applications
                    .len()
                    .saturating_mul(size_of::<Option<Predicate>>()),
            );
        work.reserve(bytes)?;
        let mut scopes = Vec::with_capacity(rows.scopes.len());
        scopes.resize_with(rows.scopes.len(), || None);
        let mut bodies = Vec::with_capacity(rows.functions.len());
        bodies.resize_with(rows.functions.len(), || None);
        Ok(Self {
            scopes,
            bodies,
            applications: vec![None; rows.trait_applications.len()],
            bytes,
        })
    }
    pub(super) fn retained_bytes(&self) -> usize {
        self.bytes
    }
    fn cached(
        &self,
        rows: s::ValidatedView<'_>,
        expected: Ready,
        origin: originals::Origin,
        work: &mut ReservationWork,
    ) -> Result<Option<Demand<()>>, Error> {
        work.charge(2)?;
        let entry = self
            .bodies
            .get(expected.pair.function_row as usize)
            .ok_or_else(|| error("private defining environment lost its actual function row"))?;
        match entry.as_deref() {
            None => Ok(None),
            Some(Body::Blocked {
                expected: actual,
                dependency,
            }) if *actual == expected => Ok(Some(Demand::Blocked(*dependency))),
            Some(Body::Ready {
                expected: actual,
                scope,
                environment,
            }) if *actual == expected && origin == originals::Origin::Declaration => {
                let declaration = self
                    .scopes
                    .get(*scope)
                    .and_then(Option::as_deref)
                    .ok_or_else(|| {
                        error("private defining environment lost its actual declaration scope")
                    })?;
                declaration.require(rows)?;
                if declaration.raw != expected.scope.1
                    || declaration.parameters.len() != expected.formals
                    || !Arc::ptr_eq(environment, &declaration.environment)
                {
                    return Err(error(
                        "private defining environment changed its retained declaration owner",
                    ));
                }
                Ok(Some(Demand::Ready(())))
            }
            Some(Body::Contract {
                expected: actual,
                scope,
                product,
            }) if *actual == expected
                && product.seal == origin
                && matches!(origin, originals::Origin::Contract(_)) =>
            {
                let declaration = self
                    .scopes
                    .get(*scope)
                    .and_then(Option::as_deref)
                    .ok_or_else(|| error("empty Contract lost its actual prepared declaration"))?;
                declaration.require(rows)?;
                declaration.require_empty(expected)?;
                if !Arc::ptr_eq(&product.declaration, &declaration.environment)
                    || Arc::ptr_eq(&product.environment, &declaration.environment)
                    || !product.environment.is_neutral()
                {
                    return Err(error(
                        "empty Contract changed its retained body construction",
                    ));
                }
                Ok(Some(Demand::Ready(())))
            }
            _ => Err(error(
                "private defining environment changed its sealed header or body",
            )),
        }
    }
    fn complete(
        &mut self,
        view: RecipeView<'_>,
        state: &mut State,
        facts: &mut Declarations,
        prepared: &Prepared,
        demand: &originals::BodyDemand,
        work: &mut ReservationWork,
    ) -> Result<Demand<()>, Error> {
        let originals::BodyDemand { expected, origin } = *demand;
        let rows = view.rows()?;
        if let Some(status) = self.cached(rows, expected, origin, work)? {
            return Ok(status);
        }
        let path = formal_path::path(rows, &prepared.scopes, expected.header.scope, work)?;
        let dependency = Dependency::OwnedBodyDomain(
            expected
                .pair
                .body
                .ok_or_else(|| error("private defining request omitted its real original body"))?,
        );
        if !originals::leaf_path(rows, &path.indices, work)? {
            self.publish(
                expected,
                Body::Blocked {
                    expected,
                    dependency,
                },
                work,
            )?;
            return Ok(Demand::Blocked(dependency));
        }
        let mut port = ports::Inputs {
            port: Reserved::new(view, &mut state.maps)?,
            cursor: &mut state.cursor,
        };
        let mut parent = None;
        for index in path.indices.into_iter().rev() {
            self.scope(rows, &mut port, facts, index, parent, work)?;
            parent = Some(index);
        }
        let index = parent
            .ok_or_else(|| error("private defining environment has no actual declaration scope"))?;
        let scope = self.scopes[index]
            .as_deref()
            .ok_or_else(|| error("private defining declaration scope disappeared"))?;
        let (environment, bytes) = originals::finish(
            facts,
            &prepared.templates,
            rows,
            &mut port,
            scope,
            demand,
            work,
        )?;
        let body = match origin {
            originals::Origin::Declaration => Body::Ready {
                expected,
                scope: index,
                environment,
            },
            originals::Origin::Contract(_) => {
                work.reserve(size_of::<ContractBody>())?;
                Body::Contract {
                    expected,
                    scope: index,
                    product: Box::new(ContractBody {
                        seal: origin,
                        declaration: Arc::clone(&scope.environment),
                        environment,
                    }),
                }
            }
        };
        let product_bytes = if matches!(origin, originals::Origin::Contract(_)) {
            size_of::<ContractBody>()
        } else {
            0
        };
        self.publish(expected, body, work)?;
        self.bytes = self
            .bytes
            .saturating_add(bytes)
            .saturating_add(product_bytes);
        Ok(Demand::Ready(()))
    }
    fn publish(
        &mut self,
        expected: Ready,
        body: Body,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(1)?;
        work.reserve(size_of::<Body>())?;
        let cell = self
            .bodies
            .get_mut(expected.pair.function_row as usize)
            .ok_or_else(|| error("private defining environment has no publication cell"))?;
        if cell.is_some() {
            return Err(error(
                "private defining environment repeated its retained publication",
            ));
        }
        *cell = Some(Box::new(body));
        self.bytes = self.bytes.saturating_add(size_of::<Body>());
        Ok(())
    }
    fn scope(
        &mut self,
        rows: s::ValidatedView<'_>,
        port: &mut ports::Inputs<'_, '_>,
        facts: &mut Declarations,
        index: usize,
        parent: Option<usize>,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(1)?;
        let raw = *rows.scopes.get(index).ok_or_else(|| {
            error("private declaration scope index is outside its immutable rows")
        })?;
        if let Some(scope) = self.scopes.get(index).and_then(Option::as_deref) {
            if scope.raw != raw {
                return Err(error("private declaration scope changed its actual row"));
            }
            scope.require(rows)?;
            return Ok(());
        }
        let inherited = parent
            .map(|index| {
                self.scopes
                    .get(index)
                    .and_then(Option::as_deref)
                    .ok_or_else(|| {
                        error("private declaration omitted its actual parent environment")
                    })
            })
            .transpose()?;
        if inherited.map(|scope| scope.raw.owner) != raw.parent {
            return Err(error(
                "private declaration changed its complete actual parent scope",
            ));
        }
        let (scope, bytes) =
            Scope::prepare(port, facts, raw, inherited, &mut self.applications, work)?;
        self.scopes[index] = Some(Box::new(scope));
        self.bytes = self.bytes.saturating_add(bytes);
        Ok(())
    }
}
struct Observed<'a> {
    work: &'a mut ReservationWork,
    allocation: Allocation,
}
impl Budget for Observed<'_> {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.work.charge(count)
    }
    fn reserve(&mut self, bytes: usize) -> Result<(), Error> {
        self.work.reserve(bytes)
    }
}
impl Accounting for Observed<'_> {
    fn observe(&mut self, allocation: Allocation) {
        self.allocation.include(allocation);
    }
}
