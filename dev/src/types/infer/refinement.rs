//! Commit isolated input refinements after closing lifetimes and validating current inputs.
mod interfaces;
use super::{
    Constructor, Context, Error, RegionId, Store, TermId, Value,
    arguments::{Argument, Slot},
    constants::ConstTermId,
    import::{Import, Mapping, Tracking},
};
use crate::support::hash::HashSet;
use crate::types::substitute::Bound;

/// An isolated query can refine its imported type/const holes. The source may
/// acquire additional constraints while the query waits. This transport grants
/// no trait or lifetime proof; the caller must retain the selected evidence.
#[derive(Default)]
pub struct Refinement {
    pub context: Context,
    inputs: Option<Box<Roots>>,
}
#[derive(Default)]
struct Roots {
    types: Vec<(TermId, TermId)>,
    constants: Vec<(ConstTermId, ConstTermId)>,
    stable: Vec<(TermId, TermId)>,
}
impl Tracking for Roots {
    fn ty(&mut self, original: TermId, copied: TermId) {
        self.types.push((original, copied));
    }
    fn constant(&mut self, original: ConstTermId, copied: ConstTermId) {
        self.constants.push((original, copied));
    }
    fn stable(&mut self, original: TermId, copied: TermId) {
        self.stable.push((original, copied));
    }
}

/// Type/const refinements do not discharge these query-scoped lifetime equations.
/// The semantic caller retains them in the same scope as the selected evidence.
#[must_use]
pub struct Committed {
    pub changed: bool,
    pub equalities: Vec<(RegionId, RegionId)>,
}

impl Refinement {
    /// Import mixed roots together, retaining source holes and immutable origins.
    /// Known types and unrelated source constraints require no snapshot copies.
    /// # Errors
    /// Rejects invalid identities and exhausted inference work.
    pub fn new(
        store: &Store,
        source: &mut Context,
        roots: &[Argument],
    ) -> Result<(Self, Vec<Argument>), Error> {
        let mut query = Self::default();
        let mut mappings = Mapping::<Roots>::default();
        let roots = import(store, source, &mut query.context, roots, &mut mappings)?;
        query.retain_inputs(mappings);
        Ok((query, roots))
    }

    fn retain_inputs(&mut self, mappings: Mapping<Roots>) {
        let roots = mappings.tracker;
        self.inputs =
            (!roots.types.is_empty() || !roots.constants.is_empty() || !roots.stable.is_empty())
                .then(|| Box::new(roots));
    }

    /// Isolated selection and its lifetime equations remain owned by the caller.
    pub const fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    /// Import a complete predicate, including mixed arguments and equality results.
    /// # Errors
    /// Rejects malformed input identities and exhausted inference work.
    pub fn predicate(
        store: &Store,
        source: &mut Context,
        predicate: &super::predicates::Predicate,
    ) -> Result<(Self, super::predicates::Predicate), Error> {
        let mut query = Self::default();
        let predicate = query.initialize_predicate(store, source, predicate)?;
        Ok((query, predicate))
    }

    /// Build directly in the owned query, avoiding an extra large-context return.
    /// An existing query's terms and input mapping cannot be replaced.
    pub(crate) fn initialize_predicate(
        &mut self,
        store: &Store,
        source: &mut Context,
        predicate: &super::predicates::Predicate,
    ) -> Result<super::predicates::Predicate, Error> {
        if self.inputs.is_some() || !self.context.terms.is_empty() {
            return Err(Error::InvalidIdentity);
        }
        let mut mappings = Mapping::<Roots>::default();
        let predicate = predicate.map_arguments(|argument| {
            self.context.account_work(1)?;
            mappings.argument(store, source, &mut self.context, argument)
        })?;
        self.retain_inputs(mappings);
        Ok(predicate)
    }

    fn roots(&self, copied: bool) -> Vec<Argument> {
        let Some(inputs) = &self.inputs else {
            return Vec::new();
        };
        inputs
            .types
            .iter()
            .map(|&(original, query)| Argument::Type(if copied { query } else { original }))
            .chain(
                inputs.constants.iter().map(|&(original, query)| {
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
        if self
            .inputs
            .as_ref()
            .is_none_or(|roots| roots.types.is_empty() && roots.constants.is_empty())
        {
            return Ok(());
        }
        self.refresh_slot_loss(source)?;
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
        let changed = self.publish(store, source, closing)?;
        Ok(Committed {
            changed,
            equalities: self.context.region_equalities,
        })
    }

    /// Publish input progress while retaining the query and its lifetime evidence.
    /// Later resumption reconciles current source inputs through the same roots.
    /// Charges only work since the last successful publication.
    /// # Errors
    /// Has the same prevalidation and failure contract as `Self::commit`.
    pub fn publish(
        &mut self,
        store: &mut Store,
        source: &mut Context,
        closing: &mut Bound,
    ) -> Result<bool, Error> {
        let Some(selected_inputs) = &self.inputs else {
            source.account_work(self.context.steps)?;
            self.context.steps = 0;
            return Ok(false);
        };
        if selected_inputs.types.is_empty() && selected_inputs.constants.is_empty() {
            source.account_work(self.context.steps)?;
            self.context.steps = 0;
            self.publish_stable(store, source)?;
            return Ok(false);
        }
        let roots = self.roots(false);
        let inputs = Inputs::discover(source, &roots)?;
        let exported = self.context.map_region_roots(
            store,
            &selected_inputs
                .types
                .iter()
                .map(|&(_, id)| id)
                .collect::<Vec<_>>(),
            closing,
        )?;
        let exported: Vec<_> = exported
            .into_iter()
            .map(Argument::Type)
            .chain(
                selected_inputs
                    .constants
                    .iter()
                    .map(|&(_, id)| Argument::Const(id)),
            )
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
        self.context.steps = 0;
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
        self.publish_slot_loss(source)?;
        source.equate_input_pairs(store, &bindings)?;
        // These equations relate already-closed inputs in the actual caller.
        // The query's original equations instead remain in its selected scope.
        source
            .region_equalities
            .append(&mut validation.region_equalities);
        self.publish_stable(store, source)?;
        Ok(changed)
    }

    /// These source graphs had no holes, projections or lifetimes at import.
    /// Reuse identities actually demanded by this query, without publishing
    /// query constraints, private universes or speculative canonical allocations.
    fn publish_stable(&mut self, store: &Store, source: &mut Context) -> Result<(), Error> {
        let Some(inputs) = &self.inputs else {
            return Ok(());
        };
        source.account_work(inputs.stable.len())?;
        for &(original, copied) in &inputs.stable {
            let copied = self.context.root(copied)?;
            let ty = match self.context.terms[copied.0 as usize].value {
                Value::Known(ty) => Some(ty),
                _ => self.context.settled_type(copied),
            };
            if let Some(ty) = ty {
                let original = source.root(original)?;
                source.remember_settled(store, original, ty)?;
            }
        }
        Ok(())
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
        let mut types = HashSet::default();
        let mut constants = HashSet::default();
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
    mappings: &mut Mapping<impl Tracking>,
) -> Result<Vec<Argument>, Error> {
    target.account_work(roots.len())?;
    roots
        .iter()
        .map(|&argument| mappings.argument(store, source, target, argument))
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
