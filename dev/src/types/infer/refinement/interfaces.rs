//! Committed body interfaces transport constraints before their receiving-body proof.
use super::{Argument, Bound, Context, Error, Import, Refinement, Store, Value, import};

impl Refinement {
    /// Import current inputs into this body's lifetime universe. The caller must
    /// normalize both sides and reconcile the returned equations in this body.
    /// No source normalization evidence crosses this interface.
    pub(crate) fn incoming_regions(
        &mut self,
        store: &mut Store,
        source: &mut Context,
        opening: &mut Bound,
    ) -> Result<Vec<(Argument, Argument)>, Error> {
        let roots = self.roots(false);
        let incoming = import(
            store,
            source,
            &mut self.context,
            &roots,
            &mut Import::default(),
        )?;
        let incoming = map_types(&mut self.context, store, incoming, opening)?;
        Ok(self.roots(true).into_iter().zip(incoming).collect())
    }

    /// Export a required body's interface without speculative query validation.
    /// Unchanged holes retain identity; fresh partial graphs share one mapping.
    /// The actual receiving body must normalize and reconcile every returned
    /// equation. Any later failure discards that body, not just this transport.
    pub(crate) fn outgoing_regions(
        &mut self,
        store: &mut Store,
        source: &mut Context,
        closing: &mut Bound,
    ) -> Result<Vec<(Argument, Argument)>, Error> {
        let roots = self.roots(true);
        let exported = map_types(&mut self.context, store, roots, closing)?;
        let mut reverse = self.unchanged_interface_holes(source)?;
        let incoming = import(store, &mut self.context, source, &exported, &mut reverse)?;
        source.account_work(self.context.steps)?;
        self.context.steps = 0;
        self.publish_stable(store, source)?;
        Ok(self.roots(false).into_iter().zip(incoming).collect())
    }

    fn unchanged_interface_holes(&mut self, source: &mut Context) -> Result<Import, Error> {
        let mut reverse = Import::default();
        let Some(inputs) = &self.inputs else {
            return Ok(reverse);
        };
        for &(original, query) in &inputs.types {
            self.context.step()?;
            let original = source.root(original)?;
            let query = self.context.root(query)?;
            if let (Value::Variable(before), Value::Variable(after)) = (
                source.terms[original.0 as usize].value,
                self.context.terms[query.0 as usize].value,
            ) && before == after
            {
                reverse.types.entry(query).or_insert(original);
            }
        }
        for &(original, query) in &inputs.constants {
            self.context.step()?;
            let original = source.constants().root(original)?;
            let query = self.context.constants().root(query)?;
            if source.constants().value(original)?.is_none()
                && self.context.constants().value(query)?.is_none()
            {
                reverse.constants.entry(query).or_insert(original);
            }
        }
        Ok(reverse)
    }
}

fn map_types(
    context: &mut Context,
    store: &mut Store,
    roots: Vec<Argument>,
    bound: &mut Bound,
) -> Result<Vec<Argument>, Error> {
    let types = roots
        .iter()
        .filter_map(|arg| match arg {
            Argument::Type(term) => Some(*term),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mapped = context.map_region_roots(store, &types, bound)?;
    let mut mapped = mapped.into_iter();
    roots
        .into_iter()
        .map(|arg| match arg {
            Argument::Type(_) => mapped
                .next()
                .map(Argument::Type)
                .ok_or(Error::InvalidIdentity),
            other => Ok(other),
        })
        .collect()
}
