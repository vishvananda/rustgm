//! Authenticate the installed declaration and recorded domain of an original initializer.
use super::{Catalog, Declarations, DefId, Graph, Mapping, Original, OriginalHeader, charge, s};
use crate::{
    resolve::crates::ReservationWork,
    sema::crates::typed_headers::{Headers, unpaid_provided::InitializerHeader},
};

impl Graph {
    pub(in crate::sema::crates::typed_headers) fn initializer_header(
        &self,
        catalog: &Catalog,
        facts: &Declarations,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<Option<InitializerHeader>, &'static str> {
        charge(steps)?;
        let descriptor = self.descriptor(catalog)?;
        let declaration = catalog.declaration(definition)?;
        let (provider, row, _) = catalog.declaration_coordinate(declaration)?;
        let rows = descriptor.rows();
        if provider != descriptor.identity()
            || !descriptor.profile().declaration_policies()
            || !matches!(rows.header(row)?, Some(s::HeaderRef::Constant(_)))
        {
            return Ok(None);
        }
        let Some((_, s::ConstantBody::Original(raw))) =
            self.associated_constant(catalog, facts, definition)?
        else {
            return Ok(None);
        };
        let index = rows.original_index(raw)?;
        let original = rows.original(raw)?;
        let input = rows.body_input(raw)?;
        let unit = rows
            .body_units
            .get(index)
            .filter(|unit| unit.body == raw)
            .ok_or("source initializer lost its actual numbering unit")?;
        let result = facts
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .ok_or("source initializer lost its canonical result")?;
        if original.declaration != row
            || original.context != s::CallContext::ClosedConst
            || input.builtins.as_ref().is_some_and(|builtins| {
                builtins.environment != s::EnvironmentOrigin::Absent
                    && builtins.environment != input.environment
            })
        {
            return Err("source initializer changed its actual declaration or defining context");
        }
        Ok(Some(InitializerHeader {
            definition,
            declaration,
            row,
            raw,
            result,
            binder: input.frame,
            body: catalog.body_at(provider, raw)?,
            universals: unit.universals,
        }))
    }

    pub(super) fn source_initializer<'a>(
        &'a self,
        catalog: &'a Catalog,
        facts: &'a Declarations,
        headers: &'a Headers,
        initializer: InitializerHeader,
        steps: &mut usize,
    ) -> Result<Original<'a>, &'static str> {
        let super::super::Canonical::Lazy(lazy) = &self.canonical else {
            return Err("source initializer requires its actual lazy declaration owner");
        };
        headers
            .charge(0)
            .map_err(|_| "source initializer has an exhausted header ledger")?;
        if headers.base.saturating_add(headers.bytes) > crate::support::crates::MAX_BYTES {
            return Err("source initializer has an exhausted header storage ledger");
        }
        let mut work = ReservationWork::default();
        let domain = lazy.initializer_body(self, facts, initializer, &mut work);
        *steps = steps.saturating_add(work.steps);
        if *steps > crate::support::crates::MAX_COMPONENTS {
            return Err("source initializer domain lookup work budget exceeded");
        }
        let domain =
            domain.map_err(|_| "source initializer requires its actual current body domain")?;
        if domain.environment().is_none() {
            return Err("source initializer omitted its actual defining environment");
        }
        let rows = self.descriptor(catalog)?.rows();
        let original = Original {
            mapping: Mapping::Ordinary {
                graph: self,
                store: &facts.types,
                domain: Some(domain),
            },
            members: &facts.members,
            catalog,
            header: OriginalHeader::Initializer(initializer),
            definition: initializer.definition,
            body: initializer.body,
            rows,
            row: u32::try_from(rows.original_index(initializer.raw)?)
                .map_err(|_| "source initializer row overflow")?,
        };
        original.check_locals(steps)?;
        Ok(original)
    }
}
