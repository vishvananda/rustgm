//! Private declaration assumptions share canonical clauses and Environment indexes.
use super::{Prepared, Progress, Ready, Session, State, formal_path, numbering_records};
#[cfg(test)]
use crate::types::parameters::predicates::Predicate;
use crate::{
    resolve::{
        crates::{ReservationWork, unpaid_provided::RecipeView},
        package::Error,
    },
    sema::{
        crates::typed_headers::{
            error,
            predicate_remap::{self, translation},
            recipe_cursor::{Budget, Cursor, Demand, Dependency, Port, Recorded, Reserved},
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
            predicates::Bound,
        },
    },
};
use std::{
    cell::{Cell, OnceCell},
    rc::Rc,
    sync::Arc,
};
use translation::applications::{Cache, Reader as ApplicationReader, Shared as ApplicationMemo};
#[cfg(test)]
#[path = "../../../../tests/invariants/inherited_environments.rs"]
mod inherited_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/remapped_inputs.rs"]
mod input_tests;
#[path = "neutral_definitions.rs"]
mod numbered;
#[path = "assumed_originals.rs"]
mod originals;
#[path = "assumption_ports.rs"]
mod ports;
#[path = "installed_inputs.rs"]
mod scope_borrows;
pub(in crate::sema::crates::typed_headers) use scope_borrows::Reader as ScopeReader;
#[cfg(test)]
#[path = "../../../../tests/invariants/assumed_environments.rs"]
mod tests;

pub(in crate::sema::crates::typed_headers) struct Declaration {
    pub index: usize,
    pub definition: crate::resolve::model::DefId,
    pub parent: Option<crate::resolve::model::DefId>,
    pub parameters: Vec<crate::sema::generics::Parameter>,
    pub environment: Arc<Environment>,
    pub inputs: Arc<[Input]>,
    pub trait_inputs: Option<predicate_remap::Prepared>,
}
struct Scope {
    raw: s::ParameterScope,
    inputs: ScopeInputs,
    parameters: Vec<(ParameterId, Abilities)>,
    declared: Vec<crate::sema::generics::Parameter>,
    installed: Cell<bool>,
    clauses: Vec<Clause>,
    environment: Arc<Environment>,
}
enum ScopeInputs {
    Trait {
        prepared: Cell<Option<predicate_remap::Prepared>>,
        all: Arc<[Input]>,
        subjects: usize,
    },
    Declaration(Arc<[Input]>),
}
impl ScopeInputs {
    fn prepare(
        port: &mut ports::Inputs<'_, '_>,
        applications: &mut impl Cache,
        scope: s::ParameterScope,
        shape: (crate::support::crates::generics::Range, Option<usize>),
        store: &mut crate::types::Store,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        if shape.1.is_some() {
            let prepared = translation::trait_inputs(port, applications, scope.owner, store, work)?;
            Self::trait_inputs(prepared, work)
        } else {
            let (inputs, _) =
                translation::inputs(port, applications, scope.owner, shape.0, None, store, work)?;
            Ok(Self::Declaration(inputs))
        }
    }
    fn trait_inputs(
        prepared: predicate_remap::Prepared,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        work.charge(1)?;
        let all = Arc::clone(prepared.all());
        let subjects = prepared.subjects().len();
        Ok(Self::Trait {
            prepared: Cell::new(Some(prepared)),
            all,
            subjects,
        })
    }
    fn all(&self) -> &Arc<[Input]> {
        match self {
            Self::Trait { all, .. } | Self::Declaration(all) => all,
        }
    }
    fn retained_subject_bytes(&self) -> usize {
        let Self::Trait {
            prepared, subjects, ..
        } = self
        else {
            return 0;
        };
        let token = prepared.take();
        let bytes = if token.is_some() { *subjects } else { 0 };
        prepared.set(token);
        bytes
    }
    fn require_untransferred(
        &self,
        declaration: crate::resolve::body_origin::ImportedDeclarationId,
    ) -> Result<(), Error> {
        if let Self::Trait { prepared, .. } = self {
            let token = prepared.take();
            let result = token
                .as_ref()
                .map_or(Ok(()), |token| token.require_declaration(declaration));
            prepared.set(token);
            result?;
        }
        Ok(())
    }
}

impl Scope {
    fn prepare(
        port: &mut ports::Inputs<'_, '_>,
        facts: &mut Declarations,
        scope_row: s::ParameterScope,
        inherited: Option<&Scope>,
        applications: &mut impl Cache,
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
                .saturating_add(slots.saturating_mul(size_of::<crate::sema::generics::Parameter>()))
                .saturating_add(
                    (previous_parameters + slots)
                        .saturating_mul(size_of::<(ParameterId, Abilities)>()),
                )
                .saturating_add((previous_clauses + own).saturating_mul(size_of::<Clause>()))
                .saturating_add(2 * size_of::<usize>()),
        )?;
        let mut parameters = Vec::with_capacity(previous_parameters + slots);
        let mut clauses = Vec::with_capacity(previous_clauses + own);
        let mut declared = Vec::with_capacity(slots);
        if let Some(parent) = inherited {
            parameters.extend_from_slice(&parent.parameters);
            clauses.extend_from_slice(&parent.clauses);
        }
        let definition = port.port.definition(scope_row.owner, work)?;
        declared_formals(
            port,
            &mut facts.types,
            &scope_row,
            definition,
            work,
            &mut parameters,
            &mut declared,
        )?;
        let inputs = ScopeInputs::prepare(
            port,
            applications,
            scope_row,
            (range, canonical),
            &mut facts.types,
            work,
        )?;
        originals::require_leaf_inputs(rows, scope_row, inputs.all(), applications, work)?;
        ports::implementation_application(
            port,
            applications,
            scope_row.owner,
            &mut facts.types,
            work,
        )?;
        clauses.extend(inputs.all().iter().map(|input| input.value));
        let mut accounting = Observed {
            work,
            allocation: Allocation::default(),
        };
        let environment = environments::prepare_checked(
            facts,
            inherited.map(|scope| &scope.environment),
            &parameters,
            &ImportedLeaves(&clauses),
            previous_parameters,
            previous_clauses,
            &mut accounting,
        )?;
        let scope = Scope {
            raw: scope_row,
            inputs,
            parameters,
            declared,
            installed: Cell::new(false),
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
            .saturating_add(retained::vector(&self.declared))
            .saturating_add(retained::vector(&self.clauses))
            .saturating_add(self.inputs.all().len().saturating_mul(size_of::<Input>()))
            .saturating_add(self.inputs.retained_subject_bytes())
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(environment)
    }
    fn require_empty(&self, expected: Ready, origin: originals::Origin) -> Result<(), Error> {
        let originals::Origin::Contract(actual) = origin else {
            return Err(error("empty Contract requires its sealed construction"));
        };
        actual.require_scope(expected.header, self.raw)?;
        if self.raw != expected.scope.1
            || expected.formals != 0
            || !self.parameters.is_empty()
            || !self.clauses.is_empty()
            || !self.inputs.all().is_empty()
            || self.inputs.retained_subject_bytes() != 0
        {
            return Err(error(
                "empty Contract changed its complete prepared declaration inputs",
            ));
        }
        Ok(())
    }
    fn subject_count(
        &self,
        rows: s::ValidatedView<'_>,
        generics: &crate::sema::generics::Generics,
        catalog: Option<&crate::resolve::body_origin::Catalog>,
    ) -> Result<usize, Error> {
        let (_, canonical) = originals::input_range(rows, self.raw)?;
        let (
            ScopeInputs::Trait {
                prepared,
                all,
                subjects,
            },
            Some(prefix),
        ) = (&self.inputs, canonical)
        else {
            return match (&self.inputs, canonical) {
                (ScopeInputs::Declaration(_), None) => Ok(0),
                _ => Err(error(
                    "private declaration environment changed its canonical input kind",
                )),
            };
        };
        if let Some(token) = prepared.take() {
            let valid = Arc::ptr_eq(token.all(), all)
                && token.subjects().len() == *subjects
                && *subjects == prefix;
            prepared.set(Some(token));
            return if valid {
                Ok(prefix)
            } else {
                Err(error(
                    "private trait inputs changed their actual producer/Arc/prefix",
                ))
            };
        }
        let catalog = catalog
            .ok_or_else(|| error("transferred trait inputs lost their actual installed catalog"))?;
        let definition = catalog
            .definition_at(rows.descriptor().identity(), self.raw.owner)
            .map_err(error)?;
        let header = generics
            .traits
            .as_deref()
            .and_then(|contracts| contracts.header(crate::types::ItemId(definition.0)))
            .ok_or_else(|| error("transferred trait inputs lost their actual canonical header"))?;
        let (actual, subjects) = header
            .inputs
            .imported_parts(catalog, definition)
            .map_err(error)?;
        if !Arc::ptr_eq(actual, all) || subjects.len() != prefix {
            return Err(error(
                "transferred trait inputs changed their actual all-input Arc or canonical prefix",
            ));
        }
        Ok(prefix)
    }
    fn require(&self, rows: s::ValidatedView<'_>, facts: &Declarations) -> Result<(), Error> {
        self.require_data(
            rows,
            &facts.generics,
            facts
                .crates
                .as_deref()
                .and_then(crate::sema::crates::Linkage::origin_catalog),
        )
    }
    fn require_data(
        &self,
        rows: s::ValidatedView<'_>,
        generics: &crate::sema::generics::Generics,
        catalog: Option<&crate::resolve::body_origin::Catalog>,
    ) -> Result<(), Error> {
        let (range, canonical) = originals::input_range(rows, self.raw)?;
        if self.inputs.all().len() != range.slice(&rows.clauses).map_err(error)?.len()
            || self.subject_count(rows, generics, catalog)? != canonical.unwrap_or(0)
        {
            return Err(error(
                "private declaration environment changed its complete canonical input owner",
            ));
        }
        Ok(())
    }
}
/// Created only after the current scope's raw effects and actual translated
/// predicates agree; the inherited prefix is copied from its sealed parent.
struct ImportedLeaves<'a>(&'a [Clause]);
impl environments::sealed::Sealed for ImportedLeaves<'_> {}
impl environments::LeafClauses for ImportedLeaves<'_> {
    fn clauses(&self) -> &[Clause] {
        self.0
    }
    fn application(&self, bound: Bound) -> Result<crate::types::traits::ApplicationId, Error> {
        match bound {
            Bound::Trait(id) | Bound::ConstTrait(id) | Bound::MaybeConstTrait(id) => Ok(id),
            _ => Err(error(
                "private leaf environment changed its checked trait effect",
            )),
        }
    }
}
#[path = "reconstruction_requests.rs"]
mod body_requests;
pub(in crate::sema::crates::typed_headers) fn ordinary_requests(
    body: crate::support::crates::owner_catalog::BodyRef,
    port: Recorded<'_, '_, '_>,
    cursor: &mut Cursor,
    applications: &mut impl Cache,
    store: &mut crate::types::Store,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    body_requests::prepare(
        body,
        &mut ports::Adapter { port, cursor },
        applications,
        store,
        work,
    )
}
#[path = "recorded_body.rs"]
mod selected_body;
pub(in crate::sema::crates::typed_headers) use selected_body::OriginalBody;

struct NumberedBody {
    recording: numbering_records::Key,
    declaration: Arc<Environment>,
    environment: Arc<Environment>,
}
struct ContractBody {
    seal: originals::Origin,
    declaration: Arc<Environment>,
    environment: Arc<Environment>,
}
enum Body {
    Numbered {
        expected: Ready,
        scope: usize,
        product: Box<NumberedBody>,
    },
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
impl Body {
    fn completed(
        origin: originals::Origin,
        expected: Ready,
        index: usize,
        finished: originals::Finished,
        scope: &Scope,
        work: &mut ReservationWork,
    ) -> Result<(Self, usize), Error> {
        let environment = finished.environment;
        match origin {
            originals::Origin::Declaration => Ok((
                Self::Ready {
                    expected,
                    scope: index,
                    environment,
                },
                0,
            )),
            originals::Origin::Contract(_) => {
                work.reserve(size_of::<ContractBody>())?;
                Ok((
                    Self::Contract {
                        expected,
                        scope: index,
                        product: Box::new(ContractBody {
                            seal: origin,
                            declaration: Arc::clone(&scope.environment),
                            environment,
                        }),
                    },
                    size_of::<ContractBody>(),
                ))
            }
            originals::Origin::Numbered => {
                work.reserve(size_of::<NumberedBody>())?;
                Ok((
                    Self::Numbered {
                        expected,
                        scope: index,
                        product: Box::new(NumberedBody {
                            recording: finished.recording.ok_or_else(|| {
                                error("neutral body lost its checked recording key")
                            })?,
                            declaration: Arc::clone(&scope.environment),
                            environment,
                        }),
                    },
                    size_of::<NumberedBody>(),
                ))
            }
        }
    }
}
pub(super) struct Owner {
    scopes: Rc<Vec<OnceCell<Box<Scope>>>>,
    bodies: Vec<Option<Box<Body>>>,
    applications: ApplicationMemo,
    bytes: usize,
}
impl Owner {
    fn new(rows: s::ValidatedView<'_>, work: &mut ReservationWork) -> Result<Self, Error> {
        work.charge(rows.scopes.len().saturating_add(rows.functions.len()))?;
        let bytes = size_of::<Self>()
            .saturating_add(size_of::<Vec<OnceCell<Box<Scope>>>>() + 2 * size_of::<usize>())
            .saturating_add(
                rows.scopes
                    .len()
                    .saturating_mul(size_of::<OnceCell<Box<Scope>>>()),
            )
            .saturating_add(
                rows.functions
                    .len()
                    .saturating_mul(size_of::<Option<Box<Body>>>()),
            );
        work.reserve(bytes)?;
        let applications = ApplicationMemo::new(rows.trait_applications.len(), work)?;
        let bytes = bytes.saturating_add(applications.retained_bytes());
        let mut scopes = Vec::with_capacity(rows.scopes.len());
        scopes.resize_with(rows.scopes.len(), OnceCell::new);
        let mut bodies = Vec::with_capacity(rows.functions.len());
        bodies.resize_with(rows.functions.len(), || None);
        Ok(Self {
            scopes: Rc::new(scopes),
            bodies,
            applications,
            bytes,
        })
    }
    pub(super) fn application_reader(
        &self,
        view: &RecipeView<'_>,
        store: &crate::types::Store,
        work: &mut ReservationWork,
    ) -> Result<ApplicationReader, Error> {
        self.applications.reader(view, store, work)
    }
    pub(super) fn require_application_reader(
        &self,
        reader: &ApplicationReader,
        view: &RecipeView<'_>,
        store: &crate::types::Store,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        self.applications.require_reader(reader, view, store, work)
    }
    fn declaration(
        &mut self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        index: usize,
        work: &mut ReservationWork,
    ) -> Result<Declaration, Error> {
        let rows = view.rows()?;
        let scope = self
            .scopes
            .get(index)
            .and_then(OnceCell::get)
            .map(Box::as_ref)
            .ok_or_else(|| error("selected declaration omitted its actual prepared scope"))?;
        scope.require(rows, facts)?;
        work.charge(scope.declared.len().saturating_add(3))?;
        work.reserve(
            scope
                .declared
                .len()
                .saturating_mul(size_of::<crate::sema::generics::Parameter>()),
        )?;
        let definition = view.definition(scope.raw.owner, work)?;
        let parent = scope
            .raw
            .parent
            .map(|owner| view.definition(owner, work))
            .transpose()?;
        let parameters = scope.declared.clone();
        let environment = Arc::clone(&scope.environment);
        let inputs = Arc::clone(scope.inputs.all());
        let is_trait = matches!(
            rows.header(scope.raw.owner).map_err(error)?,
            Some(s::HeaderRef::Trait(_))
        );
        let trait_inputs = if is_trait {
            Some(self.take_trait_inputs(view, index, work)?)
        } else {
            None
        };
        Ok(Declaration {
            index,
            definition,
            parent,
            parameters,
            environment,
            inputs,
            trait_inputs,
        })
    }
    fn take_trait_inputs(
        &mut self,
        view: &RecipeView<'_>,
        index: usize,
        work: &mut ReservationWork,
    ) -> Result<predicate_remap::Prepared, Error> {
        work.charge(4)?;
        let rows = view.rows()?;
        let raw = *rows
            .scopes
            .get(index)
            .ok_or_else(|| error("trait input transfer lost its actual dense scope row"))?;
        let scope = self
            .scopes
            .get(index)
            .and_then(OnceCell::get)
            .map(Box::as_ref)
            .ok_or_else(|| error("trait input transfer requires its actual prepared scope"))?;
        let Some(s::HeaderRef::Trait(header)) = rows.header(raw.owner).map_err(error)? else {
            return Err(error(
                "trait input transfer requires its actual trait header",
            ));
        };
        let header = &rows.traits[header as usize];
        let ScopeInputs::Trait {
            prepared: token,
            all,
            ..
        } = &scope.inputs
        else {
            return Err(error(
                "trait input ownership was already transferred or never prepared",
            ));
        };
        let prepared = token.take().ok_or_else(|| {
            error("trait inputs were already transferred or failed before publication")
        })?;
        if scope.raw != raw
            || header.scope != raw.owner
            || raw.parent != Some(header.self_definition)
            || prepared.all().len() != header.inputs.slice(&rows.clauses).map_err(error)?.len()
            || prepared.subjects().len() != header.canonical_inputs as usize
        {
            return Err(error(
                "trait input transfer changed its exact header/scope/prefix identity",
            ));
        }
        prepared.require_declaration(view.declaration(header.declaration, work)?)?;
        work.charge(1)?;
        if !Arc::ptr_eq(all, prepared.all()) {
            return Err(error("trait transfer changed its actual input Arc"));
        }
        Ok(prepared)
    }
    pub(super) fn retained_bytes(&self) -> usize {
        self.bytes
    }
    fn cached(
        &self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        prepared: &Prepared,
        demand: &originals::BodyDemand,
        work: &mut ReservationWork,
    ) -> Result<Option<Demand<()>>, Error> {
        let originals::BodyDemand { expected, origin } = *demand;
        let rows = view.rows()?;
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
            Some(
                body @ Body::Numbered {
                    expected: actual, ..
                },
            ) if *actual == expected && origin == originals::Origin::Numbered => {
                self.cached_numbered(body, view, facts, prepared, demand, work)?;
                Ok(Some(Demand::Ready(())))
            }
            Some(Body::Ready {
                expected: actual,
                scope,
                environment,
            }) if *actual == expected && origin == originals::Origin::Declaration => {
                let declaration = self
                    .scopes
                    .get(*scope)
                    .and_then(OnceCell::get)
                    .map(Box::as_ref)
                    .ok_or_else(|| {
                        error("private defining environment lost its actual declaration scope")
                    })?;
                declaration.require(rows, facts)?;
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
                    .and_then(OnceCell::get)
                    .map(Box::as_ref)
                    .ok_or_else(|| error("empty Contract lost its actual prepared declaration"))?;
                declaration.require(rows, facts)?;
                declaration.require_empty(expected, origin)?;
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
        if let Some(status) = self.cached(&view, facts, prepared, demand, work)? {
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
            port: Reserved::new(view, &state.maps, &facts.types, work)?,
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
            .get()
            .map(Box::as_ref)
            .ok_or_else(|| error("private defining declaration scope disappeared"))?;
        if origin == originals::Origin::Numbered {
            work.charge(2)?;
            if scope.environment.neutral_alias_finish_bound(0).is_err() {
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
        }
        let finished = originals::finish(facts, prepared, rows, &mut port, self, demand, work)?;
        let bytes = finished.bytes;
        let (body, product_bytes) =
            Body::completed(origin, expected, index, finished, scope, work)?;
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
        if let Some(scope) = self
            .scopes
            .get(index)
            .and_then(OnceCell::get)
            .map(Box::as_ref)
        {
            if scope.raw != raw {
                return Err(error("private declaration scope changed its actual row"));
            }
            scope.require(rows, facts)?;
            scope
                .inputs
                .require_untransferred(port.port.declaration(raw.owner, work)?)?;
            return Ok(());
        }
        let inherited = parent
            .map(|index| {
                self.scopes
                    .get(index)
                    .and_then(OnceCell::get)
                    .map(Box::as_ref)
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
        self.scopes[index]
            .set(Box::new(scope))
            .map_err(|_| error("private declaration cannot replace its actual scope cell"))?;
        self.bytes = self.bytes.saturating_add(bytes);
        Ok(())
    }
}
pub(super) fn declaration_ready(
    view: &RecipeView<'_>,
    state: &State,
    facts: &Declarations,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<bool, Error> {
    work.charge(6)?;
    let rows = view.rows()?;
    let Some(prepared) = state.callables.as_deref() else {
        return Ok(false);
    };
    let (index, raw) = prepared.scope(rows, owner)?;
    let Some(scope) = prepared
        .environments
        .as_deref()
        .and_then(|owner| owner.scopes.get(index))
        .and_then(OnceCell::get)
        .map(Box::as_ref)
    else {
        return Ok(false);
    };
    if !scope.installed.get() {
        return Ok(false);
    }
    scope.require(rows, facts)?;
    let definition = view.definition(owner, work)?;
    let parent = raw
        .parent
        .map(|parent| view.definition(parent, work))
        .transpose()?;
    if facts.generics.parents.get(&definition).copied() != parent {
        return Err(error(
            "installed declaration changed its actual normalized parent",
        ));
    }
    if scope.raw != raw
        || facts
            .generics
            .parameters
            .get(&definition)
            .is_none_or(|parameters| parameters.len() != scope.declared.len())
        || facts
            .generics
            .environments
            .get(&definition)
            .is_none_or(|environment| !Arc::ptr_eq(environment, &scope.environment))
    {
        return Err(error(
            "installed declaration changed its actual scope/formal/environment owner",
        ));
    }
    Ok(true)
}
pub(super) fn declarations(
    view: RecipeView<'_>,
    state: &mut State,
    facts: &mut Declarations,
    prepared: &mut Prepared,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<Vec<Declaration>, Error> {
    let rows = view.rows()?;
    let path = formal_path::path(rows, &prepared.scopes, owner, work)?;
    work.reserve(path.indices.len().saturating_mul(size_of::<Declaration>()))?;
    if prepared.environments.is_none() {
        prepared.environments = Some(Box::new(Owner::new(rows, work)?));
    }
    let scopes = prepared
        .environments
        .as_mut()
        .ok_or_else(|| error("selected declaration lost its scope owner"))?;
    let mut port = ports::Inputs {
        port: Reserved::new(view, &state.maps, &facts.types, work)?,
        cursor: &mut state.cursor,
    };
    let mut result = Vec::with_capacity(path.indices.len());
    let mut parent = None;
    for index in path.indices.into_iter().rev() {
        scopes.scope(rows, &mut port, facts, index, parent, work)?;
        parent = Some(index);
        let scope = scopes.scopes[index]
            .get()
            .map(Box::as_ref)
            .ok_or_else(|| error("selected declaration scope disappeared"))?;
        if scope.installed.get() {
            work.charge(scope.declared.len().saturating_add(2))?;
            let definition = port.port.definition(scope.raw.owner, work)?;
            let actual = facts
                .generics
                .parameters
                .get(&definition)
                .ok_or_else(|| error("installed declaration omitted its actual own formals"))?;
            let environment = facts
                .generics
                .environments
                .get(&definition)
                .ok_or_else(|| error("installed declaration omitted its actual environment"))?;
            if !Arc::ptr_eq(environment, &scope.environment)
                || actual.len() != scope.declared.len()
                || !actual.iter().zip(&scope.declared).all(|(a, b)| {
                    a.binding == b.binding
                        && a.argument == b.argument
                        && a.anonymous == b.anonymous
                        && a.default == b.default
                })
            {
                return Err(error(
                    "installed declaration changed its actual formals or environment",
                ));
            }
            continue;
        }
        result.push(scopes.declaration(port.port.recipe_view(), facts, index, work)?);
    }
    Ok(result)
}
pub(super) fn mark_installed(prepared: &mut Prepared, rows: &[Declaration]) -> Result<(), Error> {
    let owner = prepared
        .environments
        .as_mut()
        .ok_or_else(|| error("installed declaration lost its scope owner"))?;
    for row in rows {
        let scope = owner
            .scopes
            .get(row.index)
            .and_then(OnceCell::get)
            .map(Box::as_ref)
            .ok_or_else(|| error("installed declaration lost its actual scope cell"))?;
        if scope.installed.get() {
            return Err(error("declaration was installed twice"));
        }
        scope.installed.set(true);
    }
    Ok(())
}
fn declared_formals(
    port: &mut ports::Inputs<'_, '_>,
    store: &mut crate::types::Store,
    scope: &s::ParameterScope,
    definition: crate::resolve::model::DefId,
    work: &mut ReservationWork,
    parameters: &mut Vec<(ParameterId, Abilities)>,
    declared: &mut Vec<crate::sema::generics::Parameter>,
) -> Result<(), Error> {
    formal_path::visit(
        &mut port.port,
        store,
        scope,
        definition,
        work,
        |parameter| {
            parameters.push((parameter.parameter, Abilities::default()));
            declared.push(crate::sema::generics::Parameter {
                binding: parameter.binding,
                anonymous: false,
                argument: parameter.argument,
                default: None,
            });
        },
    )
}
pub(in crate::sema::crates::typed_headers) struct Observed<'a> {
    pub(in crate::sema::crates::typed_headers) work: &'a mut ReservationWork,
    pub(in crate::sema::crates::typed_headers) allocation: Allocation,
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
