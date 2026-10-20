//! Owned path lookup retains its prefix, candidate cursor and isolated trial across headers.
mod conditions;
use super::{
    Assumptions, Caller, Context, Declarations, DefId, Environment, Error, ItemId, NameId, NodeKey,
    Prefix, Report, ScopeId, TermId, account_probe, applicability, candidates, trait_groups,
};
use crate::{
    resolve::{model::Access, package::ModuleId},
    sema::{annotations::Scratch, traits::implementations::Availability},
};
use std::{collections::HashMap, sync::Arc};

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
    Inherent(ModuleId),
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
        bucket: (ItemId, NameId),
        scope: ScopeId,
        caller: OwnedCaller,
    ) -> Result<Self, Error> {
        let mut source = Prefix {
            report,
            facts,
            infer: Context::default(),
            terms: HashMap::new(),
        };
        let term = Scratch::default().infer(&mut source, prefix)?;
        let members = candidates(source.facts, &mut source.infer, term, bucket)
            .map_err(|message| Declarations::error(report, key, message))?;
        source.facts.methods.selection_steps = source
            .facts
            .methods
            .selection_steps
            .saturating_add(source.infer.steps);
        Ok(Self {
            prefix: source.infer,
            term,
            mode: Mode::Inherent(report.resolution.scopes[scope.0 as usize].module),
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
        site: (ScopeId, NameId),
        caller: OwnedCaller,
    ) -> Result<Self, Error> {
        let mut source = Prefix {
            report,
            facts,
            infer: Context::default(),
            terms: HashMap::new(),
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
            Mode::Inherent(module) => {
                if let Access::Within(ancestor) =
                    report.resolution.definitions[member.0 as usize].access
                    && !report.package.is_descendant(module, ancestor)
                {
                    return Ok(());
                }
                facts
                    .associated
                    .owner(member)
                    .and_then(|id| facts.associated.get(id))
                    .ok_or("missing method candidate owner")?
                    .self_definition
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
        let Some(conditions) = applicability::prepare(facts, &mut context, prefix, owner, None)?
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
