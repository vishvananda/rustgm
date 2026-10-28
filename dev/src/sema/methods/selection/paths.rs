//! Owned path lookup retains its prefix, candidate cursor and isolated trial across headers.
mod conditions;
use super::super::buckets::Bucket;
use super::{
    Assumptions, Caller, Context, Declarations, DefId, Environment, Error, NameId, NodeKey, Prefix,
    Report, ScopeId, TermId, account_probe, applicability, candidates, trait_groups,
};
use crate::sema::{annotations::Scratch, traits::implementations::Availability};
use crate::support::hash::HashMap;
use std::sync::Arc;

pub(in crate::sema) struct OwnedCaller {
    pub availability: Availability,
    pub environment: Option<Arc<Environment>>,
    pub traits: Option<Arc<Assumptions>>,
}
impl OwnedCaller {
    fn borrowed(&self) -> Caller<'_> {
        Caller {
            availability: self.availability,
            environment: self.environment.as_ref(),
            traits: self.traits.as_deref(),
        }
    }
}

pub(in crate::sema) enum Progress {
    Ready(Option<DefId>),
    Pending(crate::sema::traits::selection::Dependency),
}
enum Mode {
    Inherent(super::super::names::Use),
    Trait { bound: bool },
}
struct Trial {
    member: DefId,
    context: Context,
    conditions: applicability::Conditions,
    pending: conditions::Pending,
}
pub(in crate::sema) struct Query {
    prefix: Context,
    term: TermId,
    mode: Mode,
    members: std::vec::IntoIter<DefId>,
    lexical: Option<Vec<DefId>>,
    selected: Option<DefId>,
    trial: Option<Trial>,
    caller: OwnedCaller,
}
impl Query {
    pub fn inherent(
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
        prefix: NodeKey,
        bucket: impl Into<Bucket>,
        site: super::super::names::Use,
        caller: OwnedCaller,
    ) -> Result<Self, Error> {
        let mut source = Prefix {
            report,
            facts,
            infer: Context::default(),
            terms: HashMap::default(),
        };
        let term = Scratch::default().infer(&mut source, prefix)?;
        Self::inherent_prefix(
            report,
            source.facts,
            key,
            (source.infer, term),
            bucket,
            site,
            caller,
        )
    }

    /// A compiler protocol supplies a real canonical nominal, not fabricated syntax.
    pub fn inherent_type(
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
        ty: crate::types::TypeId,
        bucket: impl Into<Bucket>,
        site: super::super::names::Use,
        caller: OwnedCaller,
    ) -> Result<Self, Error> {
        let mut prefix = Context::default();
        let term = prefix
            .known(&facts.types, ty)
            .map_err(|error| Declarations::error(report, key, applicability::inference(error)))?;
        Self::inherent_prefix(report, facts, key, (prefix, term), bucket, site, caller)
    }

    fn inherent_prefix(
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
        (mut prefix, term): (Context, TermId),
        bucket: impl Into<Bucket>,
        site: super::super::names::Use,
        caller: OwnedCaller,
    ) -> Result<Self, Error> {
        let members = candidates(report, facts, &mut prefix, term, bucket.into(), site)
            .map_err(|message| Declarations::error(report, key, message))?
            .members;
        facts.methods.selection_steps = facts.methods.selection_steps.saturating_add(prefix.steps);
        Ok(Self {
            prefix,
            term,
            mode: Mode::Inherent(site),
            members: members.into_iter(),
            lexical: None,
            selected: None,
            trial: None,
            caller,
        })
    }

    pub fn traits(
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
        prefix: NodeKey,
        site: (ScopeId, NameId, super::super::names::Use),
        caller: OwnedCaller,
    ) -> Result<Self, Error> {
        let mut source = Prefix {
            report,
            facts,
            infer: Context::default(),
            terms: HashMap::default(),
        };
        let term = Scratch::default().infer(&mut source, prefix)?;
        let [bound, lexical] = trait_groups(
            report,
            source.facts,
            &mut source.infer,
            term,
            site,
            caller.borrowed(),
        )
        .map_err(|message| Declarations::error(report, key, message))?;
        let object = matches!(
            source.infer.head(&source.facts.types, term),
            Ok(Some(crate::types::infer::Constructor::Dynamic { .. }))
        );
        Ok(Self {
            prefix: source.infer,
            term,
            mode: Mode::Trait { bound: !object },
            members: bound.into_iter(),
            lexical: Some(lexical),
            selected: None,
            trial: None,
            caller,
        })
    }

    pub fn poll(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
    ) -> Result<Progress, &'static str> {
        loop {
            if let Some(mut trial) = self.trial.take() {
                let possible = match trial.pending.poll(
                    facts,
                    &mut trial.context,
                    &trial.conditions.traits,
                    &mut self.caller,
                )? {
                    conditions::Progress::Pending(owner) => {
                        self.trial = Some(trial);
                        return Ok(Progress::Pending(owner));
                    }
                    conditions::Progress::Ready(possible) => {
                        possible
                            && applicability::builtin_requirements(
                                facts,
                                &mut trial.context,
                                &trial.conditions.builtins,
                                self.caller.environment.as_ref(),
                            )?
                    }
                };
                account_probe(facts, trial.context.steps)?;
                if possible && self.selected.replace(trial.member).is_some() {
                    return Err(match self.mode {
                        Mode::Inherent(_) => "multiple applicable inherent methods",
                        Mode::Trait { .. } => "multiple applicable trait methods",
                    });
                }
            }
            if let Some(member) = self.members.next() {
                self.prepare(report, facts, member)?;
                continue;
            }
            if self.selected.is_some() {
                if matches!(self.mode, Mode::Trait { bound: true }) {
                    facts.methods.bound_uses.insert(key);
                }
                return Ok(Progress::Ready(self.selected));
            }
            if let Some(lexical) = self.lexical.take() {
                self.members = lexical.into_iter();
                self.mode = Mode::Trait { bound: false };
            } else {
                return Ok(Progress::Ready(None));
            }
        }
    }

    fn prepare(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        member: DefId,
    ) -> Result<(), &'static str> {
        let owner = match self.mode {
            Mode::Inherent(site) => {
                let owner = facts
                    .associated
                    .member_owner(member)
                    .ok_or("missing method candidate owner")?;
                if !site.permits(report, owner, member)? {
                    return Ok(());
                }
                owner
            }
            Mode::Trait { .. } => facts
                .associated
                .trait_owner(member)
                .ok_or("missing trait candidate owner")?,
        };
        let mut context = Context::default();
        let prefix = context
            .import(&facts.types, &mut self.prefix, self.term)
            .map_err(applicability::inference)?;
        let Some(conditions) = applicability::prepare(
            facts,
            &mut context,
            prefix,
            owner,
            None,
            self.caller.borrowed(),
        )?
        else {
            return account_probe(facts, context.steps);
        };
        let pending = conditions::Pending::new(conditions.traits.len(), facts)?;
        self.trial = Some(Trial {
            member,
            context,
            conditions,
            pending,
        });
        Ok(())
    }
}
