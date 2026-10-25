//! All impl kinds share generic/lifetime elaboration before their own conformance checks.
mod continuation;
use super::{BinderId, Checker, DefId, Error, NodeKey, Work, regions};
use crate::sema::associated::Schema;
use crate::types::parameters::{
    constraints::Constraints,
    predicates::{Bound, Predicate},
    uses::Uses,
};
pub(super) use continuation::{Pending, Phase};

pub(super) struct Header {
    pub binder: BinderId,
    pub context: regions::Context,
    pub schema: Schema,
}

struct Partial {
    binder: BinderId,
    context: regions::Context,
    bounds: Vec<crate::types::outlives::Pair>,
}

impl Checker<'_> {
    pub(super) fn reserve_implementation_candidates(&mut self) -> Result<(), Error> {
        let mut work = 0_usize;
        let result = self.implementation_candidates(&mut work);
        self.facts.body_work.canonical_steps =
            self.facts.body_work.canonical_steps.saturating_add(work);
        let headers = result?;
        if headers.is_empty() {
            return Ok(());
        }
        self.facts
            .generics
            .implementations
            .get_or_insert_with(Default::default)
            .reserve(headers)
            .map_err(Error::unlocated)?;
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::unlocated("missing reserved implementation inventory"))?
            .register_collection_heads(self.report)
            .map_err(Error::unlocated)?;
        self.prepare_imported_candidates()
    }

    fn implementation_candidates(
        &self,
        imported_work: &mut usize,
    ) -> Result<Vec<crate::sema::traits::implementations::PendingHeader>, Error> {
        use crate::sema::{associated::OwnerKind, traits::implementations::PendingHeader};
        let mut headers = Vec::new();
        let mut heads = super::heads::Heads::new(self.report, &self.facts);
        for index in 0..self.facts.associated.owners().len() {
            let actual = self.facts.associated.owners()[index];
            if matches!(actual.kind, OwnerKind::ImportedImpl(_)) {
                *imported_work = imported_work.saturating_add(1);
                let predicate = self
                    .facts
                    .associated
                    .imported_impl_request(actual)
                    .map_err(Error::unlocated)?;
                let Some((application, _)) = predicate.bound.trait_parts() else {
                    return Err(Error::unlocated(
                        "imported candidate lacks its actual trait application",
                    ));
                };
                let application =
                    self.facts
                        .types
                        .trait_application(application)
                        .ok_or_else(|| {
                            Error::unlocated("imported candidate lost its full trait arguments")
                        })?;
                let kind = self.facts.types.kind(predicate.ty).ok_or_else(|| {
                    Error::unlocated("imported candidate lacks its actual Self type")
                })?;
                headers.push(PendingHeader {
                    owner: actual.self_definition,
                    item: application.item.into(),
                    head: (!matches!(kind, crate::types::Kind::Parameter(_)))
                        .then_some(crate::sema::traits::heads::Head::canonical(kind)),
                });
                continue;
            }
            let Some(owner) = self
                .facts
                .associated
                .source_at(index)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            let OwnerKind::Impl {
                target,
                trait_type: Some(path),
                ..
            } = owner.kind
            else {
                continue;
            };
            // Builtin Drop has a separate registry. Invalid paths retain the
            // ordinary validation order and cannot supply trait candidates.
            let Ok((_, definition)) = self.resolved_impl_trait_path(path) else {
                continue;
            };
            headers.push(PendingHeader {
                owner: owner.self_definition,
                item: self.impl_trait_identity(definition),
                // Head classification is only an exclusion hint. A malformed
                // alias must retain its candidate and its later diagnostic.
                head: heads.shape(target).unwrap_or_default(),
            });
        }
        self.library_implementation_headers(&mut headers)?;
        Ok(headers)
    }

    fn prepare_imported_candidates(&mut self) -> Result<(), Error> {
        use crate::sema::{associated::OwnerKind, traits::implementations::Implementation};
        let facts = &mut self.facts;
        for &owner in facts.associated.owners() {
            if !matches!(owner.kind, OwnerKind::ImportedImpl(_)) {
                continue;
            }
            facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
            let predicate = facts
                .associated
                .imported_impl_request(owner)
                .map_err(Error::unlocated)?;
            let Some((application, _)) = predicate.bound.trait_parts() else {
                return Err(Error::unlocated(
                    "imported candidate lacks its actual full application",
                ));
            };
            let identity = facts
                .types
                .trait_application(application)
                .ok_or_else(|| {
                    Error::unlocated("imported candidate lost its actual trait identity")
                })?
                .item
                .into();
            facts
                .generics
                .implementations
                .as_mut()
                .ok_or_else(|| {
                    Error::unlocated("imported candidates lack their reserved registry")
                })?
                .prepare_imported(
                    &mut facts.types,
                    &facts.associated,
                    identity,
                    owner,
                    Implementation {
                        owner: owner.self_definition,
                        source: None,
                        predicate,
                        methods: Vec::new(),
                        types: Vec::new(),
                        constants: Vec::new(),
                    },
                )
                .map_err(Error::unlocated)?;
        }
        Ok(())
    }

    fn library_implementation_headers(
        &self,
        headers: &mut Vec<crate::sema::traits::implementations::PendingHeader>,
    ) -> Result<(), Error> {
        use crate::sema::traits::implementations::PendingHeader;
        if let Some(iteration) = self.report.resolution.builtin_traits.iteration() {
            for declaration in iteration.blankets {
                let trait_ = match declaration.kind {
                    crate::support::iteration::Blanket::Identity => {
                        crate::support::iteration::Trait::IntoIterator
                    }
                    crate::support::iteration::Blanket::MutableReference => {
                        crate::support::iteration::Trait::Iterator
                    }
                };
                headers.push(PendingHeader {
                    owner: declaration.definition,
                    item: crate::types::ItemId(iteration.get(trait_).definition.0).into(),
                    head: match declaration.kind {
                        crate::support::iteration::Blanket::Identity => None,
                        crate::support::iteration::Blanket::MutableReference => {
                            Some(crate::sema::traits::heads::Head::Reference(true))
                        }
                    },
                });
            }
        }
        if let (Some(iteration), Some(collections)) = (
            self.report.resolution.builtin_traits.iteration(),
            self.report.resolution.builtin_traits.collections(),
        ) {
            for declaration in collections.declarations() {
                if let Some(trait_) = declaration.kind.trait_() {
                    headers.push(PendingHeader {
                        owner: declaration.definition,
                        item: crate::types::ItemId(iteration.get(trait_).definition.0).into(),
                        head: Some(if declaration.kind.next() {
                            crate::sema::traits::heads::Head::Nominal(crate::types::ItemId(
                                collections
                                    .iterator(declaration.kind)
                                    .ok_or_else(|| {
                                        Error::at(
                                            collections.origin.file,
                                            0,
                                            "missing canonical collection iterator",
                                        )
                                    })?
                                    .0,
                            ))
                        } else {
                            crate::sema::traits::heads::Head::Reference(declaration.kind.mutable())
                        }),
                    });
                }
            }
        }
        if let (Some(iteration), Some(ranges)) = (
            self.report.resolution.builtin_traits.iteration(),
            self.report.resolution.builtin_traits.ranges(),
        ) {
            for declaration in ranges.implementations {
                headers.push(PendingHeader {
                    owner: declaration.definition,
                    item: crate::types::ItemId(
                        iteration
                            .get(crate::support::iteration::Trait::Iterator)
                            .definition
                            .0,
                    )
                    .into(),
                    head: Some(crate::sema::traits::heads::Head::Nominal(
                        crate::types::ItemId(ranges.nominal.0),
                    )),
                });
            }
        }
        if let (Some(cloning), Some(collections)) = (
            self.report.resolution.builtin_traits.cloning(),
            self.report.resolution.builtin_traits.collections(),
        ) && let Some(implementation) = cloning.implementation
        {
            headers.push(PendingHeader {
                owner: implementation.definition,
                item: crate::types::ItemId(cloning.definition.0).into(),
                head: Some(crate::sema::traits::heads::Head::Nominal(
                    crate::types::ItemId(collections.iterator.0),
                )),
            });
        }
        Ok(())
    }

    pub(super) fn impl_parameters(
        &self,
        owner: DefId,
        source: NodeKey,
        header: Predicate,
    ) -> Result<(), Error> {
        let error = |e| self.error(source, e);
        let types = &self.facts.types;
        let parameters = self
            .facts
            .generics
            .parameter_ids(types, owner)
            .map_err(error)?;
        let mut uses = Uses::constraining();
        uses.predicate(types, header).map_err(error)?;
        if parameters
            .iter()
            .all(|id| uses.parameters.contains(id) || uses.constants.contains(id))
        {
            return Ok(());
        }
        let mut constraints = Constraints::new(types, header, uses).map_err(error)?;
        for predicate in self.facts.generics.predicates_in_scope(owner) {
            let original = self
                .facts
                .unreduced
                .predicate(predicate.source)
                .unwrap_or(predicate.value);
            constraints
                .equality(types, original.ordinary().map_err(error)?)
                .map_err(error)?;
        }
        constraints.solve().map_err(error)?;
        if parameters.into_iter().all(|id| constraints.determines(id)) {
            return Ok(());
        }
        Err(error(
            if matches!(
                header.bound,
                Bound::Trait(_) | Bound::ConstTrait(_) | Bound::MaybeConstTrait(_)
            ) {
                "impl type and const parameters must constrain Self or the trait application"
            } else {
                "impl type and const parameters must constrain its target"
            },
        ))
    }

    pub(super) fn impl_header(
        &mut self,
        owner: DefId,
        source: NodeKey,
        target: NodeKey,
        steps: &mut usize,
        limit: usize,
    ) -> Result<Header, Error> {
        let partial = self.begin_impl_parameters(owner, source)?;
        self.drain(steps, limit)?;
        self.node_work(target.file, target.node, partial.context);
        self.drain(steps, limit)?;
        self.finish_impl_header(target, partial)
    }

    fn begin_impl_parameters(&mut self, owner: DefId, source: NodeKey) -> Result<Partial, Error> {
        let declared = self.declared_regions(source, self.item(source)?.generics)?;
        let binder = self.regions.binder(None, declared.clone());
        self.regions.associate(binder, owner);
        for parameter in declared {
            self.regions
                .named(Some(binder), parameter)
                .map_err(|e| self.error(source, e))?;
        }
        let bounds = self.function_bounds(source, binder)?;
        let context = regions::Context {
            binder: Some(binder),
            position: regions::Position::Impl,
        };
        self.declaration_parameters(owner, source, context)?;
        Ok(Partial {
            binder,
            context,
            bounds,
        })
    }

    fn finish_impl_header(&self, target: NodeKey, partial: Partial) -> Result<Header, Error> {
        let Partial {
            binder,
            context,
            bounds,
        } = partial;
        Ok(Header {
            binder,
            context,
            schema: Schema {
                self_type: self.ty(target)?,
                captures: self.regions.count(binder),
                bounds,
            },
        })
    }

    pub(super) fn complete_impl_owner(
        &mut self,
        owner: DefId,
        source: NodeKey,
        header: Header,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        self.publish_impl_owner(owner, source, header)?;
        self.drain(steps, limit)
    }

    fn publish_impl_owner(
        &mut self,
        owner: DefId,
        source: NodeKey,
        header: Header,
    ) -> Result<(), Error> {
        let target = header.schema.self_type;
        self.facts
            .associated
            .declare(owner, header.schema)
            .map_err(|e| self.error(source, e))?;
        self.owner_binders.insert(owner, header.binder);
        self.complete(owner, source, target)?;
        self.work
            .push(Work::ProjectionEnvironment(owner, header.context, false));
        Ok(())
    }
}
