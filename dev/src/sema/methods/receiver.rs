//! Receiver search probes each dereference and borrow before committing equations.
use super::selection::{self, Caller};
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, DefId, MethodCall},
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
    let (base, depth) = source
        .reference_target(&facts.types, term)
        .map_err(inference)
        .map_err(error)?;
    let head = source
        .head(&facts.types, base)
        .map_err(inference)
        .map_err(error)?;
    let inherent = match head {
        Some(Constructor::Adt(nominal, _) | Constructor::AdtArgs(nominal, _))
            if facts.methods.inherent.contains_key(&(nominal, site.name)) =>
        {
            selection::candidates(facts, source, base, (nominal, site.name)).map_err(error)?
        }
        _ => Vec::new(),
    };
    let has_traits = facts.methods.trait_names.contains(&site.name);
    let library = report
        .resolution
        .library_types
        .slice_length()
        .filter(|l| report.resolution.definitions[l.definition.0 as usize].name == site.name)
        .filter(|_| array(head) || matches!(head, Some(Constructor::Slice)))
        .map(|l| l.definition)
        .or_else(|| {
            report
                .resolution
                .builtin_traits
                .collections()
                .map(|c| {
                    c.declarations[crate::support::iteration::Collection::SliceMethod as usize]
                })
                .filter(|c| report.resolution.definitions[c.method.0 as usize].name == site.name)
                .filter(|_| array(head) || matches!(head, Some(Constructor::Slice)))
                .map(|c| c.method)
        })
        .or_else(|| match head {
            Some(Constructor::Pointer { mutable }) => {
                super::pointers::named(report, site.name, mutable)
            }
            _ => None,
        });
    if inherent.is_empty() && !has_traits && library.is_none() {
        return Err(error(
            if matches!(head, Some(Constructor::Adt(..) | Constructor::AdtArgs(..))) {
                "no inherent method with this name"
            } else {
                "receiver has no inherent nominal method"
            },
        ));
    }
    let traits = if has_traits {
        selection::trait_groups(report, facts, source, base, (site.scope, site.name), caller)
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
        inherent,
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
impl Search<'_> {
    fn at(
        &mut self,
        term: TermId,
        dereferences: u32,
        unsize: bool,
    ) -> Result<Option<Selected>, &'static str> {
        for borrow in [None, Some(false), Some(true)] {
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
        let module = self.report.resolution.scopes[self.site.scope.0 as usize].module;
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
            if matches!(lane, Lane::Inherent)
                && references != self.depth - dereferences + u32::from(borrow.is_some())
            {
                continue;
            }
            if let Access::Within(ancestor) =
                self.report.resolution.definitions[method.0 as usize].access
                && !self.report.package.is_descendant(module, ancestor)
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
        if !super::slices::receiver(self.facts, self.source, term)? {
            return Ok(false);
        }
        if let Some(declaration) = self
            .report
            .resolution
            .builtin_traits
            .collections()
            .map(|c| c.declarations[crate::support::iteration::Collection::SliceMethod as usize])
            .filter(|d| d.method == method)
        {
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
