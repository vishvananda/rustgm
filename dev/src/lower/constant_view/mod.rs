//! Bounded inspection of completed values. Rendering never starts semantic queries.
mod graph;
mod roots;
use crate::borrow::Checked;
use crate::sema::model::Declarations;
use std::io::{self, Write};

/// Deterministic logical bounds for display preparation and owned rows.
#[derive(Clone, Copy)]
pub struct Limits {
    pub nodes: usize,
    pub work: usize,
    pub bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            nodes: 1_000_000,
            work: 4_000_000,
            bytes: 64 * 1024 * 1024,
        }
    }
}
/// An owned display inventory contains no borrowed proof or private arena identities.
pub struct Inventory {
    declarations: Vec<String>,
    roots: Vec<String>,
    values: Vec<String>,
    allocations: Vec<String>,
}
/// Build a deterministic graph from checked source facts, before writing any output.
/// # Errors
/// Rejects missing completed values, inconsistent stored types and bounded graph limits.
pub fn inspect(facts: &Declarations, checked: &Checked<'_>) -> Result<Inventory, &'static str> {
    inspect_with_limits(facts, checked, Limits::default())
}
/// Inspect using explicit display budgets; these cannot grant constant execution permission.
/// # Errors
/// Reports the same semantic graph errors and configured resource limits as `inspect`.
pub fn inspect_with_limits(
    facts: &Declarations,
    checked: &Checked<'_>,
    limits: Limits,
) -> Result<Inventory, &'static str> {
    graph::build(facts, checked, limits)
}
impl Inventory {
    /// # Errors
    /// Propagates output errors.
    pub fn count(&self, out: &mut impl Write) -> io::Result<()> {
        writeln!(
            out,
            "declarations={} roots={} values={} allocations={}",
            self.declarations.len(),
            self.roots.len(),
            self.values.len(),
            self.allocations.len()
        )
    }
    /// # Errors
    /// Propagates output errors.
    pub fn write(&self, out: &mut impl Write) -> io::Result<()> {
        writeln!(out, "constants 1")?;
        for row in self
            .declarations
            .iter()
            .chain(&self.roots)
            .chain(&self.values)
            .chain(&self.allocations)
        {
            writeln!(out, "{row}")?;
        }
        write!(out, "eof ")?;
        self.count(out)
    }
}
