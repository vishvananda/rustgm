//! Clause terms share normalization work while retaining their universal proof scope.
use super::{Declarations, State};
use crate::{
    sema::{
        projections::{Cursor, Progress},
        traits::implementations::Availability,
    },
    types::{RegionId, TypeId, lifetime_proof::ScopeId, substitute::Bound},
};
use std::collections::HashMap;

pub(super) struct Terms {
    owner: (u32, u32),
    close: Bound,
    cursors: HashMap<(u32, bool), Cursor>,
}
impl Terms {
    fn new(owner: (u32, u32)) -> Result<Self, &'static str> {
        let (count, first) = owner;
        let mut replacements = HashMap::new();
        for slot in 0..count {
            let index = u16::try_from(slot).map_err(|_| "too many declaration lifetimes")?;
            let region = first
                .checked_add(slot)
                .filter(|id| *id != 0 && *id < 0x8000_0000)
                .ok_or("invalid declaration lifetime interval")?;
            replacements.insert(RegionId(region), RegionId::bound(0, index)?);
        }
        Ok(Self {
            owner,
            close: Bound::restore_regions(replacements),
            cursors: HashMap::new(),
        })
    }
}

impl State {
    pub(in crate::sema::nominals) fn normalize_term(
        &mut self,
        facts: &mut Declarations,
        original: TypeId,
        count: u32,
        owner: (u32, u32),
    ) -> Result<TypeId, &'static str> {
        let fold = self
            .fold
            .as_mut()
            .ok_or("clause normalization requires an explicit declaration scope")?;
        let before = fold.steps;
        // Leave clause locals bound and open only the enclosing owner's captures.
        let opened = fold.apply_inside(&mut facts.types, original, 1)?;
        self.context
            .account_work(fold.steps.saturating_sub(before).saturating_add(1))
            .map_err(|_| "clause normalization input budget exceeded")?;
        if self.terms.is_none() {
            self.context
                .account_work(owner.0 as usize)
                .map_err(|_| "clause normalization owner budget exceeded")?;
            self.terms = Some(Box::new(Terms::new(owner)?));
        }
        let terms = self
            .terms
            .as_mut()
            .ok_or("missing clause normalization state")?;
        if terms.owner != owner {
            return Err("clause normalization changed its declaration scope");
        }
        let key = (count, matches!(self.availability, Availability::Validated));
        if let std::collections::hash_map::Entry::Vacant(entry) = terms.cursors.entry(key) {
            self.context
                .account_work(count as usize + 1)
                .map_err(|_| "clause normalization scope budget exceeded")?;
            // Conjunctive universal terms add no local hypotheses. Equal-arity
            // terms may share a universe, but each selection owns fresh witnesses.
            let scope = self.obligations.scope(ScopeId::ROOT, count)?;
            entry.insert(Cursor::new(
                self.environment.clone(),
                &self.obligations,
                scope,
                self.availability,
            )?);
        }
        let cursor = terms
            .cursors
            .get_mut(&key)
            .ok_or("missing clause normalization cursor")?;
        cursor.start(facts, opened)?;
        let (ty, steps) = match cursor
            .advance(facts, &mut self.obligations, &mut |_, _| Ok(None))?
        {
            Progress::Complete { ty, steps } => (ty, steps),
            Progress::Definition(_) => return Err("missing completed associated type definition"),
            Progress::Header(_) => {
                return Err("projection selection requires a prepared implementation header");
            }
        };
        self.context
            .account_work(steps)
            .map_err(|_| "clause normalization work budget exceeded")?;
        // Cursor restoration already removes every private selection identity.
        // Close only the immutable owner aliases, underneath the clause binder;
        // rescanning the accumulated proof for each growing root is quadratic.
        let before = terms.close.steps;
        let result = terms.close.apply_inside(&mut facts.types, ty, 1)?;
        self.context
            .account_work(terms.close.steps.saturating_sub(before))
            .map_err(|_| "clause normalization restoration budget exceeded")?;
        Ok(result)
    }
}
