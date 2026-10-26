//! A canonical callable path is borrowed structure, never an alias or paid proof.
use super::super::{Declaration, DefinitionKind};
use crate::support::crates::owner_catalog::DeclarationRef;

pub(crate) struct CallablePath<'a> {
    declarations: &'a [Declaration],
    trail: [DeclarationRef; 64],
    count: usize,
    bytes: usize,
}
impl CallablePath<'_> {
    pub(crate) fn storage_bound(&self) -> usize {
        self.count
            .saturating_mul(4 * size_of::<String>())
            .saturating_add(self.bytes.saturating_mul(4))
    }
    pub(crate) fn materialize(self) -> Result<Vec<String>, &'static str> {
        let mut result = Vec::with_capacity(self.count);
        for row in self.trail[..self.count].iter().rev() {
            result.push(
                self.declarations[row.0 as usize]
                    .name
                    .as_ref()
                    .ok_or("canonical callable name disappeared")?
                    .clone(),
            );
        }
        Ok(result)
    }
}

/// Charge the bounded scratch initialization and every emitted edge/name byte.
/// The returned recipe allocates nothing; consumers prepay its output then clone
/// each component once. Work is linear in emitted components and name bytes.
pub(crate) fn callable_path<E>(
    declarations: &[Declaration],
    root: DeclarationRef,
    start: DeclarationRef,
    mut charge: impl FnMut(usize) -> Result<(), E>,
    error: impl Fn(&'static str) -> E,
) -> Result<CallablePath<'_>, E> {
    charge(1)?;
    charge(64)?;
    let mut path = CallablePath {
        declarations,
        trail: [start; 64],
        count: 0,
        bytes: 0,
    };
    let mut current = start;
    loop {
        charge(1)?;
        let row = declarations
            .get(current.0 as usize)
            .ok_or_else(|| error("canonical callable coordinate is outside its rows"))?;
        if current == root {
            if path.count == 0
                || row.definition.kind != DefinitionKind::Module
                || row.definition.parent.is_some()
            {
                return Err(error("canonical callable changed its actual root"));
            }
            return Ok(path);
        }
        let valid_kind = if path.count == 0 {
            row.definition.kind == DefinitionKind::Function
        } else {
            row.definition.kind == DefinitionKind::Module
                || (path.count == 1 && row.definition.kind == DefinitionKind::Trait)
        };
        if !valid_kind || path.count == path.trail.len() {
            return Err(error(
                "canonical callable owner kind/depth requires a wider profile",
            ));
        }
        let name = row
            .name
            .as_ref()
            .filter(|name| !name.is_empty())
            .ok_or_else(|| error("canonical callable owner has no actual name"))?;
        charge(name.len().saturating_mul(2))?;
        path.bytes = path.bytes.saturating_add(name.len());
        path.trail[path.count] = current;
        path.count += 1;
        current = row
            .definition
            .parent
            .ok_or_else(|| error("canonical callable did not reach its authenticated root"))?;
    }
}
