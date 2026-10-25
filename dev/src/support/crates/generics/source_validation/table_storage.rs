//! Count all source columns and nested capacities before allocating owner indexes.
use super::{HeaderIndex, HeaderRef, SourceRows, Work};
use crate::support::{crates, retained};

pub(super) struct Budget {
    pub rows: usize,
    pub scope_base: usize,
    pub peak: usize,
}
struct Count {
    bytes: usize,
    components: usize,
}
impl Count {
    fn vector<T>(&mut self, values: &Vec<T>, work: &mut Work) -> Result<(), &'static str> {
        work.charge(1)?;
        self.bytes = self.bytes.saturating_add(retained::vector(values));
        self.components = self.components.saturating_add(values.len());
        self.check()
    }
    fn check(&self) -> Result<(), &'static str> {
        if self.bytes > crates::MAX_BYTES || self.components > crates::MAX_COMPONENTS {
            return Err("source columns exceed aggregate storage or component budget");
        }
        Ok(())
    }
}

pub(super) fn preflight(
    rows: &SourceRows,
    base: usize,
    work: &mut Work,
) -> Result<Budget, &'static str> {
    work.charge(1)?;
    if rows.declarations.is_empty()
        || rows.declarations.len() > crates::MAX_FUNCTIONS
        || rows.functions.len() > crates::MAX_FUNCTIONS
        || rows.body_owners.len() > crates::MAX_FUNCTIONS
    {
        return Err("source declaration or body table exceeds its row budget");
    }
    let mut count = Count {
        bytes: base.saturating_add(size_of::<SourceRows>()),
        components: 0,
    };
    columns(rows, &mut count, work)?;
    let mut names = 0_usize;
    for row in &rows.declarations {
        work.charge(1)?;
        if let Some(name) = &row.name {
            names = names.saturating_add(name.len());
            count.bytes = count.bytes.saturating_add(name.capacity());
        }
    }
    if names > crates::MAX_NAME_BYTES {
        return Err("source declaration names exceed the combined byte budget");
    }
    for body in &rows.originals {
        work.charge(1)?;
        original(body, &mut count, work)?;
    }
    for input in &rows.body_inputs {
        work.charge(1)?;
        inputs(input, &mut count, work)?;
    }
    for unit in &rows.body_units {
        work.charge(1)?;
        count.vector(&unit.allocation_order, work)?;
        if let Some(aliases) = &unit.aliases {
            count.vector(aliases, work)?;
        }
    }
    count.check()?;
    // Dense header ledger plus declaration validation's definitions/ordinals/
    // tri-colour walk and body coverage. ScopeIndex independently checks its
    // concurrent retained and transient allocations against this same base.
    let scratch = rows
        .declarations
        .len()
        .saturating_mul(size_of::<super::super::Definition>() + 3 * size_of::<u32>() + 1)
        .saturating_add(rows.body_owners.len().saturating_mul(2 * size_of::<u32>()))
        .saturating_add(count.components.saturating_mul(2));
    let ledger = rows
        .declarations
        .len()
        .saturating_mul(size_of::<Option<HeaderRef>>());
    let scope_base = count
        .bytes
        .saturating_add(size_of::<HeaderIndex<'_>>())
        .saturating_add(scratch)
        .saturating_add(ledger);
    let peak = scope_base
        .saturating_add(size_of::<super::super::ScopeIndex<'_>>())
        .saturating_add(
            rows.declarations
                .len()
                .saturating_mul(2 * size_of::<Option<u32>>()),
        )
        .saturating_add(rows.scopes.len().saturating_mul(4 * size_of::<u32>()));
    if peak > crates::MAX_BYTES {
        return Err("source owner indexes exceed the combined storage budget");
    }
    Ok(Budget {
        rows: count.bytes,
        scope_base,
        peak,
    })
}

fn columns(rows: &SourceRows, count: &mut Count, work: &mut Work) -> Result<(), &'static str> {
    count.vector(&rows.declarations, work)?;
    count.vector(&rows.body_owners, work)?;
    count.vector(&rows.slots, work)?;
    count.vector(&rows.scopes, work)?;
    count.vector(&rows.binders, work)?;
    count.vector(&rows.types, work)?;
    count.vector(&rows.roots, work)?;
    count.vector(&rows.regions, work)?;
    count.vector(&rows.arguments, work)?;
    count.vector(&rows.clauses, work)?;
    count.vector(&rows.clause_indices, work)?;
    count.vector(&rows.region_bounds, work)?;
    count.vector(&rows.region_slots, work)?;
    count.vector(&rows.trait_applications, work)?;
    count.vector(&rows.function_applications, work)?;
    count.vector(&rows.traits, work)?;
    count.vector(&rows.implementations, work)?;
    count.vector(&rows.functions, work)?;
    count.vector(&rows.formals, work)?;
    count.vector(&rows.members, work)?;
    count.vector(&rows.member_links, work)?;
    count.vector(&rows.roles, work)?;
    count.vector(&rows.originals, work)?;
    count.vector(&rows.body_inputs, work)?;
    count.vector(&rows.body_units, work)
}
fn original(
    body: &super::super::Original,
    count: &mut Count,
    work: &mut Work,
) -> Result<(), &'static str> {
    count.vector(&body.variables, work)?;
    count.vector(&body.locals, work)?;
    count.vector(&body.parameter_bindings, work)?;
    count.vector(&body.projections, work)?;
    count.vector(&body.values, work)?;
    count.vector(&body.statements, work)?;
    count.vector(&body.blocks, work)?;
    count.vector(&body.scopes, work)?;
    count.vector(&body.exits, work)?;
    count.vector(&body.temporaries, work)?;
    if let Some(owned) = &body.owned {
        count.vector(&owned.created, work)?;
        count.vector(&owned.returned, work)?;
    }
    count.vector(&body.arguments, work)?;
    count.vector(&body.openings, work)?;
    count.vector(&body.calls, work)
}
fn inputs(
    input: &super::super::BodyInputs,
    count: &mut Count,
    work: &mut Work,
) -> Result<(), &'static str> {
    count.vector(&input.parameters, work)?;
    count.vector(&input.assumptions, work)?;
    count.vector(&input.type_assumptions, work)?;
    count.vector(&input.obligations, work)?;
    count.vector(&input.normalizations, work)?;
    for row in &input.normalizations {
        work.charge(1)?;
        count.vector(&row.sources, work)?;
        count.vector(&row.owned, work)?;
        count.vector(&row.equalities, work)?;
        count.vector(&row.outlives, work)?;
        count.vector(&row.types, work)?;
    }
    if let Some(builtins) = &input.builtins {
        count.vector(&builtins.predicates, work)?;
        if let Some(copy) = &builtins.source_copy {
            count.vector(&copy.goals, work)?;
        }
    }
    Ok(())
}
