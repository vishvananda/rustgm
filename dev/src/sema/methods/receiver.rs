//! Receiver search probes each dereference and borrow before committing equations.
use super::selection::{self, Caller};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, MethodCall},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    types::{
        RegionId,
        infer::{Constructor, Context, TermId},
    },
};

#[derive(Clone, Copy)]
pub(in crate::sema) struct Selected {
    pub method: DefId,
    pub dereferences: u32,
    pub borrow: Option<bool>,
    pub self_term: Option<TermId>,
    pub unsize: bool,
    pub from_bound: bool,
}
#[derive(Clone, Copy)]
enum Lane {
    Inherent,
    Library,
    Bound,
    Lexical,
}
struct Search<'a> {
    report: &'a Report,
    facts: &'a mut Declarations,
    source: &'a mut Context,
    site: MethodCall,
    caller: Caller<'a>,
    base: TermId,
    depth: u32,
    inherent: Vec<DefId>,
    library: Option<DefId>,
    traits: [Vec<DefId>; 2],
    object: bool,
}

pub(in crate::sema) fn choose(
    report: &Report,
    facts: &mut Declarations,
    source: &mut Context,
    key: NodeKey,
    mut term: TermId,
    site: MethodCall,
    caller: Caller<'_>,
) -> Result<Selected, Error> {
    let error = |message: &str| Declarations::error(report, key, message);
    let identifier = super::names::Use::receiver(report, key, site)?;
    let (base, depth) = source
        .reference_target(&facts.types, term)
        .map_err(inference)
        .map_err(error)?;
    let head = source
        .head(&facts.types, base)
        .map_err(inference)
        .map_err(error)?;
    let inherent = inherent_candidates(report, facts, source, (term, base), head, site, identifier)
        .map_err(error)?;
    let has_traits = facts.methods.trait_names.contains(&site.name);
    let library = library_method(report, head, site.name);
    if inherent.members.is_empty() && !has_traits && library.is_none() {
        return Err(error(if inherent.filtered {
            "no applicable inherent method for this receiver"
        } else if matches!(head, Some(Constructor::Adt(..) | Constructor::AdtArgs(..))) {
            "no inherent method with this name"
        } else {
            "receiver has no inherent nominal method"
        }));
    }
    let traits = if has_traits {
        selection::trait_groups(
            report,
            facts,
            source,
            base,
            (site.scope, site.name, identifier),
            caller,
        )
        .map_err(error)?
    } else {
        [Vec::new(), Vec::new()]
    };
    let mut search = Search {
        report,
        facts,
        source,
        site,
        caller,
        base,
        depth,
        inherent: inherent.members,
        library,
        traits,
        object: matches!(head, Some(Constructor::Dynamic { .. })),
    };
    for dereferences in 0..=depth {
        if let Some(selected) = search.at(term, dereferences, false).map_err(error)? {
            return Ok(selected);
        }
        if dereferences < depth {
            term = search
                .source
                .child(&search.facts.types, term, 0)
                .map_err(inference)
                .map_err(error)?;
        }
    }
    if array(head) {
        let element = search
            .source
            .child(&search.facts.types, base, 0)
            .map_err(inference)
            .map_err(error)?;
        let slice = search
            .source
            .construct(Constructor::Slice, &[element])
            .map_err(inference)
            .map_err(error)?;
        if let Some(selected) = search.at(slice, depth, true).map_err(error)? {
            return Ok(selected);
        }
    }
    Err(error(if has_traits {
        "no applicable method for this receiver"
    } else {
        "no applicable inherent method for this receiver"
    }))
}
fn inherent_candidates(
    report: &Report,
    facts: &mut Declarations,
    source: &mut Context,
    (term, base): (TermId, TermId),
    head: Option<Constructor>,
    site: MethodCall,
    identifier: super::names::Use,
) -> Result<selection::Found, &'static str> {
    let mut inherent = match head {
        Some(Constructor::Adt(nominal, _) | Constructor::AdtArgs(nominal, _))
            if facts.methods.inherent.contains_key(&(nominal, site.name)) =>
        {
            selection::candidates(
                report,
                facts,
                source,
                base,
                (nominal, site.name),
                identifier,
            )?
        }
        _ => selection::Found::default(),
    };
    super::adjusted_targets::collect(
        report,
        facts,
        source,
        (term, base),
        site.name,
        identifier,
        &mut inherent,
    )?;
    Ok(inherent)
}
fn library_method(
    report: &Report,
    head: Option<Constructor>,
    name: crate::resolve::names::NameId,
) -> Option<DefId> {
    report
        .resolution
        .library_types
        .slice_length()
        .filter(|l| report.resolution.definitions[l.definition.0 as usize].name == name)
        .filter(|_| array(head) || matches!(head, Some(Constructor::Slice)))
        .map(|l| l.definition)
        .or_else(|| {
            report
                .resolution
                .builtin_traits
                .collections()
                .and_then(|c| {
                    c.declarations()
                        .find(|d| {
                            d.kind.method()
                                && report.resolution.definitions[d.method.0 as usize].name == name
                        })
                        .copied()
                })
                .filter(|c| report.resolution.definitions[c.method.0 as usize].name == name)
                .filter(|_| array(head) || matches!(head, Some(Constructor::Slice)))
                .map(|c| c.method)
        })
        .or_else(|| match head {
            Some(Constructor::Pointer { mutable }) => super::pointers::named(report, name, mutable),
            _ => None,
        })
}
impl Search<'_> {
    fn at(
        &mut self,
        term: TermId,
        dereferences: u32,
        unsize: bool,
    ) -> Result<Option<Selected>, &'static str> {
        for borrow in [None, Some(false), Some(true)] {
            if unsize && borrow.is_none() {
                continue;
            }
            let actual = if let Some(mutable) = borrow {
                self.source
                    .construct(
                        Constructor::Reference {
                            mutable,
                            region: RegionId::STATIC,
                        },
                        &[term],
                    )
                    .map_err(inference)?
            } else {
                term
            };
            for lane in [Lane::Inherent, Lane::Library, Lane::Bound, Lane::Lexical] {
                if let Some(method) = self.select(actual, dereferences, borrow, lane)? {
                    return Ok(Some(Selected {
                        method,
                        dereferences,
                        borrow,
                        unsize,
                        from_bound: matches!(lane, Lane::Bound) && !self.object,
                        self_term: matches!(lane, Lane::Inherent).then_some(self.base),
                    }));
                }
            }
        }
        Ok(None)
    }
    fn candidates(&self, lane: Lane) -> &[DefId] {
        match lane {
            Lane::Inherent => &self.inherent,
            Lane::Library => self.library.as_slice(),
            Lane::Bound => &self.traits[0],
            Lane::Lexical => &self.traits[1],
        }
    }
    fn select(
        &mut self,
        term: TermId,
        dereferences: u32,
        borrow: Option<bool>,
        lane: Lane,
    ) -> Result<Option<DefId>, &'static str> {
        let mut selected = None;
        for index in 0..self.candidates(lane).len() {
            let method = self.candidates(lane)[index];
            if matches!(lane, Lane::Library) {
                let possible = self.library_candidate(term, method)?;
                if possible && selected.replace(method).is_some() {
                    return Err("multiple applicable library methods");
                }
                continue;
            }
            let Some(&(receiver, references)) = self.facts.methods.receivers.get(&method) else {
                continue;
            };
            let owner_self = self
                .facts
                .associated
                .schema(method)
                .map(|schema| schema.self_type);
            let nominal = owner_self.is_some_and(|ty| {
                matches!(self.facts.types.kind(ty), Some(crate::types::Kind::Adt(..)))
            });
            if matches!(lane, Lane::Inherent)
                && nominal
                && references != self.depth - dereferences + u32::from(borrow.is_some())
            {
                continue;
            }
            let owner = if let Some(owner) = self.facts.associated.trait_owner(method) {
                if !self.possible_trait(owner, receiver, references, term)? {
                    continue;
                }
                owner
            } else {
                self.facts
                    .associated
                    .member_owner(method)
                    .ok_or("missing receiver candidate owner")?
            };
            let mut probe = Context::default();
            let actual = probe
                .import(&self.facts.types, self.source, term)
                .map_err(inference)?;
            let subject = if matches!(lane, Lane::Bound) {
                Some(
                    probe
                        .import(&self.facts.types, self.source, self.base)
                        .map_err(inference)?,
                )
            } else if matches!(lane, Lane::Inherent) && !nominal {
                let mut subject = actual;
                let mut possible = true;
                for _ in 0..references {
                    if !matches!(
                        probe.head(&self.facts.types, subject).map_err(inference)?,
                        Some(Constructor::Reference { .. })
                    ) {
                        possible = false;
                        break;
                    }
                    subject = probe
                        .child(&self.facts.types, subject, 0)
                        .map_err(inference)?;
                }
                if !possible {
                    continue;
                }
                Some(subject)
            } else {
                None
            };
            let possible = selection::applicable(
                self.facts,
                &mut probe,
                actual,
                owner,
                Some(selection::Receiver {
                    ty: receiver,
                    subject,
                }),
                self.caller,
            )?;
            selection::account_probe(self.facts, probe.steps)?;
            if possible && selected.replace(method).is_some() {
                return Err("multiple applicable methods for this receiver");
            }
        }
        Ok(selected)
    }

    fn library_candidate(&mut self, term: TermId, method: DefId) -> Result<bool, &'static str> {
        if self
            .report
            .resolution
            .intrinsics
            .identify(method)
            .is_some_and(crate::support::intrinsics::Intrinsic::is_pointer_offset)
        {
            let Some(Constructor::Pointer { mutable }) = self
                .source
                .head(&self.facts.types, term)
                .map_err(inference)?
            else {
                return Ok(false);
            };
            if super::pointers::named(self.report, self.site.name, mutable) != Some(method) {
                return Ok(false);
            }
            super::pointers::prepare(self.report, self.facts, method)?;
            selection::account_probe(self.facts, 1)?;
            return Ok(true);
        }
        let declaration = self
            .report
            .resolution
            .builtin_traits
            .collections()
            .and_then(|c| {
                c.declarations()
                    .find(|d| d.kind.method() && d.method == method)
                    .copied()
            });
        if !super::slices::receiver_permission(
            self.facts,
            self.source,
            term,
            declaration.is_some_and(|d| d.kind.mutable()),
        )? {
            return Ok(false);
        }
        if let Some(declaration) = declaration {
            super::super::declarations::iteration::collections::prepare(
                self.report,
                self.facts,
                declaration.definition,
                matches!(
                    self.caller.availability,
                    crate::sema::traits::implementations::Availability::Validated
                ),
            )
            .map_err(|_| "canonical slice iteration preparation failed")?;
        }
        let collection = self
            .facts
            .iteration
            .as_deref()
            .and_then(|r| r.collections.as_deref())
            .and_then(|r| r.original(method));
        if collection.is_none() {
            super::slices::prepare(self.report, self.facts)?;
        }
        let mut probe = Context::default();
        let actual = probe
            .import(&self.facts.types, self.source, term)
            .map_err(inference)?;
        let possible = if let Some(original) = collection {
            let receiver = self
                .facts
                .methods
                .receivers
                .get(&method)
                .ok_or("missing collection receiver")?
                .0;
            selection::applicable(
                self.facts,
                &mut probe,
                actual,
                original.declaration.definition,
                Some(selection::Receiver {
                    ty: receiver,
                    subject: None,
                }),
                self.caller,
            )?
        } else {
            super::slices::applicable(self.facts, &mut probe, actual, method)?
        };
        selection::account_probe(self.facts, probe.steps)?;
        Ok(possible)
    }
    fn possible_trait(
        &mut self,
        owner: DefId,
        mut receiver: crate::types::TypeId,
        references: u32,
        mut subject: TermId,
    ) -> Result<bool, &'static str> {
        for _ in 0..references {
            let Some(crate::types::Kind::Reference {
                mutable, referent, ..
            }) = self.facts.types.kind(receiver)
            else {
                return Err("missing trait receiver reference shape");
            };
            match self
                .source
                .head(&self.facts.types, subject)
                .map_err(inference)?
            {
                Some(Constructor::Reference {
                    mutable: actual, ..
                }) if actual == mutable => {
                    subject = self
                        .source
                        .child(&self.facts.types, subject, 0)
                        .map_err(inference)?;
                    receiver = referent;
                }
                Some(_) => return Ok(false),
                None => return Ok(true),
            }
        }
        crate::sema::traits::selection::may_have_subject(
            self.facts,
            self.source,
            crate::types::ItemId(owner.0),
            subject,
            self.caller.traits,
        )
        .map_err(inference)
    }
}
fn inference(_: crate::types::infer::Error) -> &'static str {
    "invalid or exhausted receiver inference graph"
}

fn array(head: Option<Constructor>) -> bool {
    matches!(
        head,
        Some(Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_))
    )
}
