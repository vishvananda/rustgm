//! Return a selected query's symbolic result without importing constraints on its inputs.
use super::{
    Constructor, Context, Domain, Error, Store, TermId, Value, arguments::Slot,
    constants::ConstTermId, import::Import,
};
use std::collections::HashSet;

/// One isolated query over a fixed source graph. This is an inference transport,
/// not implementation-selection or bound evidence. Keep the source's equations
/// unchanged until export; fresh query variables cannot escape into that source.
pub struct Probe {
    context: Context,
    mappings: Import,
    variables: Vec<(TermId, Domain)>,
    constants: Vec<ConstTermId>,
}

impl Probe {
    /// Copy shared input roots into an isolated context, retaining their identity
    /// for a later export. Ordinary untracked imports do not pay this bookkeeping.
    ///
    /// # Errors
    /// Rejects malformed inputs, compact overflow and exhausted inference work.
    pub fn new(
        store: &Store,
        source: &mut Context,
        roots: &[TermId],
    ) -> Result<(Self, Vec<TermId>), Error> {
        let mut context = Context::default();
        let mut mappings = Import::default();
        context.account_work(roots.len())?;
        let roots = roots
            .iter()
            .map(|&root| context.import_with(store, source, root, &mut mappings))
            .collect::<Result<Vec<_>, _>>()?;
        context.account_work(
            mappings
                .types
                .len()
                .saturating_add(mappings.constants.len()),
        )?;
        let variables = mappings
            .types
            .keys()
            .filter_map(|&id| match source.terms[id.0 as usize].value {
                Value::Variable(domain) => Some((id, domain)),
                _ => None,
            })
            .collect();
        let mut constants = Vec::new();
        for &id in mappings.constants.keys() {
            if source.constants().value(id)?.is_none() {
                constants.push(id);
            }
        }
        Ok((
            Self {
                context,
                mappings,
                variables,
                constants,
            },
            roots,
        ))
    }

    /// The caller charges this context's complete work to its owning query and
    /// handles lifetime equations separately from exporting type structure.
    pub const fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    /// Check input preservation without allocating or exporting a result graph.
    /// This proves only that no source hole was bound, narrowed or merged.
    ///
    /// # Errors
    /// Rejects changed source equations, malformed identities and exhausted work.
    pub fn preserves_inputs(&mut self, source: &mut Context) -> Result<bool, Error> {
        self.source_unchanged(source)?;
        self.inputs_preserved()
    }

    /// Reuse original holes in a query result, without committing any input
    /// refinement. Returns `None` if a query bound, narrowed or merged source
    /// holes, or if its result contains a fresh unbound query variable.
    ///
    /// Validation precedes import, so a rejected export allocates no source terms.
    /// Lifetime equations and semantic requirements are deliberately not copied.
    ///
    /// # Errors
    /// Rejects changed source equations, malformed identities and exhausted work.
    pub fn export(
        &mut self,
        store: &Store,
        source: &mut Context,
        result: TermId,
    ) -> Result<Option<TermId>, Error> {
        if !self.preserves_inputs(source)? {
            return Ok(None);
        }
        let mut reverse = Import::default();
        for (&original, &copied) in &self.mappings.types {
            let copied = self.context.root(copied)?;
            reverse.types.insert(copied, original);
        }
        for (&original, &copied) in &self.mappings.constants {
            let copied = self.context.constants().root(copied)?;
            reverse.constants.insert(copied, original);
        }
        if !self.exportable(store, result, &reverse)? {
            return Ok(None);
        }
        source
            .import_with(store, &mut self.context, result, &mut reverse)
            .map(Some)
    }

    fn source_unchanged(&mut self, source: &mut Context) -> Result<(), Error> {
        self.context.account_work(
            self.mappings.types.len() + self.mappings.constants.len() + self.variables.len(),
        )?;
        for &id in self.mappings.types.keys() {
            if source.root(id)? != id {
                return Err(Error::InvalidIdentity);
            }
        }
        for &(id, domain) in &self.variables {
            if !matches!(source.terms[id.0 as usize].value, Value::Variable(d) if d == domain) {
                return Err(Error::InvalidIdentity);
            }
        }
        for &id in self.mappings.constants.keys() {
            if source.constants().root(id)? != id {
                return Err(Error::InvalidIdentity);
            }
        }
        for &id in &self.constants {
            self.context.step()?;
            if source.constants().value(id)?.is_some() {
                return Err(Error::InvalidIdentity);
            }
        }
        Ok(())
    }

    fn inputs_preserved(&mut self) -> Result<bool, Error> {
        let mut variables = HashSet::new();
        for &(original, domain) in &self.variables {
            self.context.step()?;
            let copied = self.context.root(self.mappings.types[&original])?;
            if !matches!(self.context.terms[copied.0 as usize].value,
                Value::Variable(d) if d == domain)
                || !variables.insert(copied)
            {
                return Ok(false);
            }
        }
        let mut constants = HashSet::new();
        for &original in &self.constants {
            self.context.step()?;
            let copied = self
                .context
                .constants()
                .root(self.mappings.constants[&original])?;
            if self.context.constants().value(copied)?.is_some() || !constants.insert(copied) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn exportable(&mut self, store: &Store, root: TermId, reverse: &Import) -> Result<bool, Error> {
        let mut work = vec![root];
        let mut seen = HashSet::new();
        while let Some(id) = work.pop() {
            self.context.step()?;
            let id = self.context.root(id)?;
            if reverse.types.contains_key(&id) || !seen.insert(id) {
                continue;
            }
            let term = self.context.terms[id.0 as usize];
            let kind = match term.value {
                Value::Variable(_) => return Ok(false),
                Value::Known(ty) => {
                    store.kind(ty).ok_or(Error::InvalidIdentity)?;
                    continue;
                }
                Value::Construct(kind) => kind,
            };
            if let Constructor::ArrayInfer(length) = kind
                && !self.exportable_const(length, reverse)?
            {
                return Ok(false);
            }
            if let Some((arguments, _)) = kind.application() {
                let count = self.context.slots(arguments)?.len();
                self.context.account_work(count)?;
                for index in 0..count {
                    if let Slot::Const(id) = self.context.slots(arguments)?[index]
                        && !self.exportable_const(id, reverse)?
                    {
                        return Ok(false);
                    }
                }
            }
            self.context
                .account_work((term.end - term.start) as usize)?;
            work.extend_from_slice(&self.context.edges[term.start as usize..term.end as usize]);
        }
        Ok(true)
    }
    fn exportable_const(&mut self, id: ConstTermId, reverse: &Import) -> Result<bool, Error> {
        self.context.step()?;
        let id = self.context.constants().root(id)?;
        Ok(reverse.constants.contains_key(&id) || self.context.constants().value(id)?.is_some())
    }
}
