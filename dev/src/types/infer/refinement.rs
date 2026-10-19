//! Commit isolated input refinements after closing lifetimes and validating current inputs.
use super::{
    Constructor, Context, Error, RegionId, Store, TermId, Value,
    arguments::{Argument, Slot},
    constants::ConstTermId,
    import::Import,
};
use crate::types::substitute::Bound;
use std::collections::HashSet;

/// An isolated query can refine its imported type/const holes. The source may
/// acquire additional constraints while the query waits. This transport grants
/// no trait or lifetime proof; the caller must retain the selected evidence.
pub struct Refinement {
    context: Context,
    types: Vec<(TermId, TermId)>,
    constants: Vec<(ConstTermId, ConstTermId)>,
}

/// Type/const refinements do not discharge these query-scoped lifetime equations.
/// The semantic caller retains them in the same scope as the selected evidence.
#[must_use]
pub struct Committed {
    pub changed: bool,
    pub equalities: Vec<(RegionId, RegionId)>,
}

impl Refinement {
    /// Import mixed roots together, retaining only the reachable source holes.
    /// Known types and unrelated source constraints require no snapshot copies.
    /// # Errors
    /// Rejects invalid identities and exhausted inference work.
    pub fn new(
        store: &Store,
        source: &mut Context,
        roots: &[Argument],
    ) -> Result<(Self, Vec<Argument>), Error> {
        let inputs = Inputs::discover(source, roots)?;
        let mut context = Context::default();
        let mut mappings = Import::default();
        let roots = import(store, source, &mut context, roots, &mut mappings)?;
        let types = inputs
            .types
            .into_iter()
            .map(|id| Ok((id, *mappings.types.get(&id).ok_or(Error::InvalidIdentity)?)))
            .collect::<Result<_, Error>>()?;
        let constants = inputs
            .constants
            .into_iter()
            .map(|id| {
                Ok((
                    id,
                    *mappings.constants.get(&id).ok_or(Error::InvalidIdentity)?,
                ))
            })
            .collect::<Result<_, Error>>()?;
        Ok((
            Self {
                context,
                types,
                constants,
            },
            roots,
        ))
    }

    /// Isolated selection and its lifetime equations remain owned by the caller.
    pub const fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    fn roots(&self, copied: bool) -> Vec<Argument> {
        self.types
            .iter()
            .map(|&(original, query)| Argument::Type(if copied { query } else { original }))
            .chain(
                self.constants.iter().map(|&(original, query)| {
                    Argument::Const(if copied { query } else { original })
                }),
            )
            .collect()
    }

    /// Reconcile current source constraints before resuming an isolated query.
    /// No source equations are changed. A failed reconciliation discards the query.
    /// # Errors
    /// Rejects contradictory inputs, invalid identities and exhausted work.
    pub fn refresh(&mut self, store: &Store, source: &mut Context) -> Result<(), Error> {
        let roots = self.roots(false);
        let incoming = import(
            store,
            source,
            &mut self.context,
            &roots,
            &mut Import::default(),
        )?;
        let pairs: Vec<_> = self.roots(true).into_iter().zip(incoming).collect();
        self.context.equate_input_pairs(store, &pairs)
    }

    /// Close all exported lifetime identities, then reconcile with current inputs
    /// in an isolated validation graph before allocating or binding source terms.
    /// Unchanged holes keep their identities; fresh partial holes remain shared.
    /// Charges the isolated work to the source on success. Callers charge discarded
    /// queries separately and retain all lifetime equations outside this transport.
    /// # Errors
    /// Invalid lifetime closure or contradictory inputs leave source equations and
    /// allocations unchanged. A later resource error may retain commit progress,
    /// like ordinary inference: the caller must discard that body context.
    pub fn commit(
        mut self,
        store: &mut Store,
        source: &mut Context,
        closing: &mut Bound,
    ) -> Result<Committed, Error> {
        let roots = self.roots(false);
        let inputs = Inputs::discover(source, &roots)?;
        let exported = self.context.map_region_roots(
            store,
            &self.types.iter().map(|&(_, id)| id).collect::<Vec<_>>(),
            closing,
        )?;
        let exported: Vec<_> = exported
            .into_iter()
            .map(Argument::Type)
            .chain(self.constants.iter().map(|&(_, id)| Argument::Const(id)))
            .collect();
        let mut validation = Context::default();
        let mut source_map = Import::default();
        let current = import(store, source, &mut validation, &roots, &mut source_map)?;
        let selected = import(
            store,
            &mut self.context,
            &mut validation,
            &exported,
            &mut Import::default(),
        )?;
        let pairs: Vec<_> = current.into_iter().zip(selected).collect();
        validation.equate_input_pairs(store, &pairs)?;
        let mut reverse = inputs.preserved(source, &mut validation, &source_map)?;
        source.account_work(self.context.steps.saturating_add(validation.steps))?;
        let mut bindings = Vec::with_capacity(inputs.types.len() + inputs.constants.len());
        for original in inputs.types {
            let id = *source_map
                .types
                .get(&original)
                .ok_or(Error::InvalidIdentity)?;
            let value = source.import_with(store, &mut validation, id, &mut reverse)?;
            bindings.push((Argument::Type(original), Argument::Type(value)));
        }
        for original in inputs.constants {
            let id = *source_map
                .constants
                .get(&original)
                .ok_or(Error::InvalidIdentity)?;
            let value = reverse.constant(store, &mut validation, source, id)?;
            bindings.push((Argument::Const(original), Argument::Const(value)));
        }
        let mut changed = false;
        for &(original, selected) in &bindings {
            changed |= different(source, original, selected)?;
        }
        source.equate_input_pairs(store, &bindings)?;
        // These equations relate already-closed inputs in the actual caller.
        // The query's original equations instead remain in its selected scope.
        source
            .region_equalities
            .append(&mut validation.region_equalities);
        Ok(Committed {
            changed,
            equalities: self.context.region_equalities,
        })
    }
}

#[derive(Default)]
struct Inputs {
    types: Vec<TermId>,
    constants: Vec<ConstTermId>,
}
impl Inputs {
    fn discover(context: &mut Context, roots: &[Argument]) -> Result<Self, Error> {
        context.account_work(roots.len())?;
        let mut result = Self::default();
        let mut types = HashSet::new();
        let mut constants = HashSet::new();
        let mut pending: Vec<_> = roots.iter().rev().copied().collect();
        while let Some(root) = pending.pop() {
            context.step()?;
            let id = match root {
                Argument::Lifetime(_) => continue,
                Argument::Const(id) => {
                    let id = context.constants().root(id)?;
                    if constants.insert(id) && context.constants().value(id)?.is_none() {
                        result.constants.push(id);
                    }
                    continue;
                }
                Argument::Type(id) => context.root(id)?,
            };
            if !types.insert(id) {
                continue;
            }
            let term = context.terms[id.0 as usize];
            let kind = match term.value {
                Value::Known(_) => continue,
                Value::Variable(_) => {
                    result.types.push(id);
                    continue;
                }
                Value::Construct(kind) => kind,
            };
            if let Constructor::ArrayInfer(id) = kind {
                pending.push(Argument::Const(id));
            }
            if let Some((arguments, _)) = kind.application() {
                context.account_work(context.slots(arguments)?.len())?;
                for slot in context.slots(arguments)?.iter().rev() {
                    if let Slot::Const(id) = slot {
                        pending.push(Argument::Const(*id));
                    }
                }
            }
            context.account_work((term.end - term.start) as usize)?;
            pending.extend(
                context.edges[term.start as usize..term.end as usize]
                    .iter()
                    .rev()
                    .map(|&id| Argument::Type(id)),
            );
        }
        Ok(result)
    }

    fn preserved(
        &self,
        source: &mut Context,
        validation: &mut Context,
        mappings: &Import,
    ) -> Result<Import, Error> {
        let mut reverse = Import::default();
        for &original in &self.types {
            validation.step()?;
            let copied = validation.root(
                *mappings
                    .types
                    .get(&original)
                    .ok_or(Error::InvalidIdentity)?,
            )?;
            if let (Value::Variable(before), Value::Variable(after)) = (
                source.terms[original.0 as usize].value,
                validation.terms[copied.0 as usize].value,
            ) && before == after
            {
                reverse.types.entry(copied).or_insert(original);
            }
        }
        for &original in &self.constants {
            validation.step()?;
            let copied = validation.constants().root(
                *mappings
                    .constants
                    .get(&original)
                    .ok_or(Error::InvalidIdentity)?,
            )?;
            if validation.constants().value(copied)?.is_none() {
                reverse.constants.entry(copied).or_insert(original);
            }
        }
        Ok(reverse)
    }
}

fn import(
    store: &Store,
    source: &mut Context,
    target: &mut Context,
    roots: &[Argument],
    mappings: &mut Import,
) -> Result<Vec<Argument>, Error> {
    target.account_work(roots.len())?;
    roots
        .iter()
        .map(|&argument| {
            Ok(match argument {
                Argument::Lifetime(region) => Argument::Lifetime(region),
                Argument::Type(id) => {
                    Argument::Type(target.import_with(store, source, id, mappings)?)
                }
                Argument::Const(id) => {
                    Argument::Const(mappings.constant(store, source, target, id)?)
                }
            })
        })
        .collect()
}

fn different(context: &mut Context, left: Argument, right: Argument) -> Result<bool, Error> {
    match (left, right) {
        (Argument::Type(a), Argument::Type(b)) => Ok(context.root(a)? != context.root(b)?),
        (Argument::Const(a), Argument::Const(b)) => {
            Ok(context.constants().root(a)? != context.constants().root(b)?)
        }
        _ => Err(Error::InvalidIdentity),
    }
}
