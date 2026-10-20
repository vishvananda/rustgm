//! Stable constant eligibility retains temporary ownership and static extension.
use super::{
    Declarations, Error, Expression, Kind, NodeKey, Report, constructors, products, references,
};
use crate::{
    resolve::model::{DefKind, Target},
    sema::scopes,
    types::destruction::NeedsDrop,
};
use std::collections::HashMap;

pub(super) struct Scratch {
    scopes: scopes::Scratch,
    needs_drop: NeedsDrop,
    declared_owners: usize,
    places: HashMap<NodeKey, Option<NodeKey>>,
}
pub(super) struct Check {
    substitution: Option<crate::types::substitute::Bound>,
    scopes: Option<scopes::Facts>,
    root: NodeKey,
    scratch: Box<Scratch>,
    steps: usize,
}
impl Check {
    pub(super) fn new(
        facts: &Declarations,
        root: NodeKey,
        scratch: Option<Box<Scratch>>,
        substitution: Option<crate::types::substitute::Bound>,
    ) -> Option<Self> {
        let count = facts.destructors.declared().len();
        if count == 0 {
            return None;
        }
        let mut scratch = scratch.unwrap_or_else(|| {
            Box::new(Scratch {
                scopes: scopes::Scratch::default(),
                needs_drop: NeedsDrop::new(facts.destructors.declared()),
                declared_owners: count,
                places: HashMap::new(),
            })
        });
        // Early impl-target constants already see all reserved owners. Keep the
        // reusable scratch valid if a caller extends the declaration registry.
        if scratch.declared_owners != count {
            scratch.needs_drop = NeedsDrop::new(facts.destructors.declared());
            scratch.declared_owners = count;
        }
        Some(Self {
            substitution,
            scopes: None,
            root,
            scratch,
            steps: 0,
        })
    }
    pub(super) fn finish(mut self) -> Box<Scratch> {
        if let Some(scopes) = self.scopes {
            let (_, scratch) = scopes.finish();
            self.scratch.scopes = scratch;
        }
        self.scratch
    }
    pub(super) fn expression(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
        expression: Expression,
    ) -> Result<(), Error> {
        match expression {
            Expression::Reference(
                references::Expression::Borrow(value) | references::Expression::Address(value),
            ) => {
                self.place(report, facts, value)?;
            }
            Expression::Reference(
                references::Expression::Field { value, .. }
                | references::Expression::Index { value, .. },
            ) if facts.autoderefs.get(&key).copied().unwrap_or(0) == 0 => {
                self.place(report, facts, value)?;
            }
            Expression::Constructor(constructors::Expression::Record {
                base: Some(base), ..
            }) => {
                // Stable Rust rejects cleanup of a partially moved temporary,
                // including an update that transfers all of its Drop fields.
                self.temporary(report, facts, base)?;
            }
            Expression::Product(products::Expression::Repeat { value, length: 0 })
                if !facts.constant_repeats.contains(&key) =>
            {
                self.temporary(report, facts, value)?;
            }
            _ => {}
        }
        Ok(())
    }
    fn place(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
    ) -> Result<(), Error> {
        let mut current = key;
        let mut visited = Vec::new();
        let root;
        loop {
            if let Some(&cached) = self.scratch.places.get(&current) {
                root = cached;
                break;
            }
            self.steps += 1;
            if self.steps > 1_000_000 || self.scratch.places.len() >= 1_000_000 {
                return Err(Declarations::error(
                    report,
                    key,
                    "constant place work budget exceeded",
                ));
            }
            visited.push(current);
            match Declarations::kind(report, current) {
                Kind::Group(value) | Kind::Attributed { value, .. } => current.node = value,
                Kind::Field { value, .. } | Kind::Index { value, .. } => {
                    if facts.autoderefs.get(&current).copied().unwrap_or(0) != 0 {
                        root = None;
                        break;
                    }
                    current.node = value;
                }
                Kind::Unary {
                    op: crate::syntax::ast::Unary::Dereference,
                    ..
                } => {
                    root = None;
                    break;
                }
                Kind::Name(_) | Kind::Path { .. } | Kind::Qualified { .. }
                    if facts
                        .node(current)
                        .and_then(|f| f.reference)
                        .is_some_and(|target| {
                            matches!(target, Target::Definition(id)
                            if report.resolution.definitions[id.0 as usize].kind == DefKind::Static)
                        }) =>
                {
                    root = None;
                    break;
                }
                _ => {
                    root = Some(current);
                    break;
                }
            }
        }
        for visited in visited {
            self.scratch.places.insert(visited, root);
        }
        if let Some(root) = root {
            self.temporary(report, facts, root)?;
        }
        Ok(())
    }
    fn temporary(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
    ) -> Result<(), Error> {
        let error = |message| Declarations::error(report, key, message);
        let mut ty = facts
            .node(key)
            .and_then(|f| f.ty)
            .ok_or_else(|| error("missing constant type"))?;
        if let Some(substitution) = &mut self.substitution {
            ty = substitution.apply(&mut facts.types, ty).map_err(error)?;
            facts
                .aggregates
                .prepare(&mut facts.types, ty)
                .map_err(error)?;
        }
        if !self
            .scratch
            .needs_drop
            .demand(&facts.types, &facts.aggregates, ty)
            .map_err(error)?
        {
            return Ok(());
        }
        if self.scopes.is_none() {
            self.scopes = Some(scopes::constant_with_scratch(
                report,
                facts,
                self.root,
                std::mem::take(&mut self.scratch.scopes),
            )?);
        }
        let scope = self
            .scopes
            .as_ref()
            .and_then(|scopes| scopes.node(key.node))
            .ok_or_else(|| error("missing constant scope"))?;
        if scope.temporary == scopes::Lifetime::Static {
            Ok(())
        } else {
            Err(error("constant expression cannot destroy this temporary"))
        }
    }
}
