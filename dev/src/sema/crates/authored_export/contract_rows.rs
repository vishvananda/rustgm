//! Input rows retain actual source roles and qualifiers, never legacy permissions.
use super::{DeclarationRef, Declarations, DefId, Stage, canonical_nodes::Context, g, s};
use crate::{
    resolve::unit::NodeKey,
    support::{bindings::LangItem, constness::Constness, hash::HashMap, retained},
    types::{
        GenericArgs, TypeId,
        arguments::Argument,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
        traits::ApplicationId,
    },
};

#[derive(Default)]
pub(super) struct Index {
    sites: HashMap<(DeclarationRef, NodeKey), u32>,
    applications: HashMap<(Predicate, Context), s::TraitApplicationRef>,
    pub(super) regions: HashMap<(DeclarationRef, u32), s::SlotRef>,
}

impl Stage {
    pub(super) fn site(
        &mut self,
        owner: DeclarationRef,
        source: NodeKey,
    ) -> Result<u32, &'static str> {
        self.charge(1)?;
        if let Some(&site) = self.inputs.sites.get(&(owner, source)) {
            return Ok(site);
        }
        self.reserve(retained::new_map::<(DeclarationRef, NodeKey), u32>(1))?;
        let row = self
            .rows
            .declarations
            .get_mut(owner.0 as usize)
            .ok_or("source input site has no actual declaration")?;
        let site = row.sites;
        row.sites = site
            .checked_add(1)
            .ok_or("source declaration site overflow")?;
        self.inputs.sites.insert((owner, source), site);
        Ok(site)
    }

    pub(super) fn clause(
        &mut self,
        facts: &Declarations,
        input: crate::sema::declaration_inputs::Input,
        context: Context,
        self_subject: Option<s::SlotRef>,
    ) -> Result<u32, &'static str> {
        self.charge(1)?;
        let source = input
            .source
            .source()
            .ok_or("source export input lacks actual source provenance")?;
        let site = self.site(context.owner, source)?;
        let (predicate, binder, inner) = match input.value {
            Clause::Predicate(predicate) => (predicate, None, context),
            Clause::ForAll(id) => {
                let actual = facts
                    .types
                    .quantified(id)
                    .ok_or("source clause binder is unknown")?;
                let inner = self.frame(
                    context.owner,
                    context.frame,
                    s::BinderKind::Clause,
                    actual.bound_regions,
                )?;
                (actual.predicate, Some(self.binder(inner)?), inner)
            }
        };
        let requirement = self.requirement(facts, predicate, inner)?;
        self.reserve(4 * size_of::<s::Clause>())?;
        let id =
            u32::try_from(self.rows.clauses.len()).map_err(|_| "source clause table overflow")?;
        self.rows.clauses.push(s::Clause {
            owner: context.owner,
            site,
            binder,
            requirement,
            self_subject,
        });
        Ok(id)
    }

    pub(crate) fn requirement(
        &mut self,
        facts: &Declarations,
        predicate: Predicate,
        context: Context,
    ) -> Result<s::Requirement, &'static str> {
        self.charge(1)?;
        if let Some((application, effect)) = predicate.bound.trait_parts() {
            return Ok(s::Requirement::Trait {
                application: self.trait_application(facts, predicate.ty, application, context)?,
                effect,
            });
        }
        match predicate.bound {
            Bound::Outlives(region) => Ok(s::Requirement::TypeOutlives {
                subject: self.ty(facts, predicate.ty, context)?,
                shorter: self.region(region, context)?,
            }),
            Bound::Copy | Bound::Clone | Bound::Sized => {
                let role = match predicate.bound {
                    Bound::Copy => LangItem::Copy,
                    Bound::Clone => LangItem::Clone,
                    _ => LangItem::Sized,
                };
                let declaration = facts
                    .source_bindings
                    .as_deref()
                    .ok_or("source builtin clause has no role catalog")?
                    .header_role(role)?
                    .ok_or("source builtin clause has no actual role declaration")?;
                let subject = self.ty(facts, predicate.ty, context)?;
                let declaration = self.address(declaration.definition)?;
                let arguments = self.arguments(facts, GenericArgs::EMPTY, context)?;
                let application = self.push_trait(s::TraitApplication {
                    declaration,
                    subject,
                    arguments,
                })?;
                Ok(s::Requirement::Trait {
                    application,
                    effect: Constness::Ordinary,
                })
            }
            _ => Err("source requirement form has no complete supported transport row"),
        }
    }

    pub(super) fn trait_application(
        &mut self,
        facts: &Declarations,
        subject: TypeId,
        application: ApplicationId,
        context: Context,
    ) -> Result<s::TraitApplicationRef, &'static str> {
        self.charge(1)?;
        let key = (
            Predicate {
                ty: subject,
                bound: Bound::Trait(application),
            },
            context,
        );
        if let Some(&id) = self.inputs.applications.get(&key) {
            return Ok(id);
        }
        let actual = facts
            .types
            .trait_application(application)
            .ok_or("source trait application is unknown")?;
        let declaration = self.address(DefId(actual.item.0))?;
        let subject = self.ty(facts, subject, context)?;
        let arguments = self.arguments(facts, actual.arguments, context)?;
        let id = self.push_trait(s::TraitApplication {
            declaration,
            subject,
            arguments,
        })?;
        self.reserve(retained::new_map::<
            (Predicate, Context),
            s::TraitApplicationRef,
        >(1))?;
        self.inputs.applications.insert(key, id);
        Ok(id)
    }

    pub(crate) fn address(&mut self, actual: DefId) -> Result<s::DeclarationAddress, &'static str> {
        Ok(s::DeclarationAddress {
            artifact: s::ArtifactRef(0),
            declaration: self.declaration(actual)?,
        })
    }

    fn push_trait(
        &mut self,
        application: s::TraitApplication,
    ) -> Result<s::TraitApplicationRef, &'static str> {
        self.reserve(4 * size_of::<s::TraitApplication>())?;
        let id = s::TraitApplicationRef(
            u32::try_from(self.rows.trait_applications.len())
                .map_err(|_| "source trait application table overflow")?,
        );
        self.rows.trait_applications.push(application);
        Ok(id)
    }

    pub(crate) fn arguments(
        &mut self,
        facts: &Declarations,
        arguments: GenericArgs,
        context: Context,
    ) -> Result<g::Range, &'static str> {
        let actual = facts
            .types
            .arguments(arguments)
            .ok_or("source argument list is unknown")?;
        // Children can append their own application lists. Stage this complete
        // list before appending, so its wire range never absorbs nested inputs.
        self.reserve(actual.len().saturating_mul(8 * size_of::<s::Argument>()))?;
        let mut completed = Vec::with_capacity(actual.len());
        for argument in actual {
            self.charge(1)?;
            let value = match argument {
                Argument::Type(ty) => s::Argument::Type(self.ty(facts, ty, context)?),
                Argument::Lifetime(region) => s::Argument::Region(self.region(region, context)?),
                Argument::Const(_) => {
                    return Err("source const argument transport is not implemented");
                }
            };
            completed.push(value);
        }
        let start = u32::try_from(self.rows.arguments.len())
            .map_err(|_| "source argument table overflow")?;
        self.charge(completed.len())?;
        self.rows.arguments.extend(completed);
        Ok(g::Range {
            start,
            end: u32::try_from(self.rows.arguments.len())
                .map_err(|_| "source argument table overflow")?,
        })
    }
}
