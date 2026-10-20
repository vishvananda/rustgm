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
    if inherent.is_empty() && !has_traits {
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
    if matches!(head, Some(Constructor::Array(_))) {
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
            for lane in [Lane::Inherent, Lane::Bound, Lane::Lexical] {
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
                    .owner(method)
                    .and_then(|id| self.facts.associated.get(id))
                    .ok_or("missing receiver candidate owner")?
                    .self_definition
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
