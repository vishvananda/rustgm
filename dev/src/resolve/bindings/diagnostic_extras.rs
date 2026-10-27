//! Inert diagnostic strings share source ownership, never compiler-role permission.
use super::{
    annotations::{self, DiagnosticItem},
    policy::Feature,
};
use crate::{
    resolve::{
        model::Resolution,
        package::{Error, Package},
        unit::{FileId, NodeKey, Unit},
    },
    support::{
        crates::{
            self,
            generics::{Work, canonical_order},
        },
        names::{NameId, Names},
        retained::{self, Growth},
    },
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::{Forest, Span},
    },
};
use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub(super) struct Entry {
    pub name: NameId,
    pub owner: NodeKey,
    pub attribute: Span,
}
pub(super) struct Column {
    pub rows: Vec<Entry>,
}
struct Pending {
    name: Box<str>,
    owner: NodeKey,
    attribute: Span,
}
#[derive(Default)]
struct Budget {
    work: Work,
    bytes: usize,
    name_bytes: usize,
    failed: bool,
}
impl Budget {
    fn pay(&mut self, growth: Growth) -> Result<(), &'static str> {
        self.bytes = self.bytes.saturating_add(growth.bytes);
        let work = self.work.charge(growth.work);
        if self.failed || work.is_err() || self.bytes > crates::MAX_BYTES {
            self.failed = true;
            return Err("diagnostic data exceeds work/storage budget");
        }
        Ok(())
    }
    fn names(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.name_bytes = self.name_bytes.saturating_add(bytes);
        if self.name_bytes > crates::MAX_NAME_BYTES {
            self.failed = true;
        }
        self.pay(Growth {
            work: bytes,
            bytes: 0,
        })
    }
}
#[derive(Default)]
pub(super) struct Draft {
    pending: Vec<Pending>,
    foreign: Option<Vec<NodeKey>>,
    budget: Budget,
    error_site: Option<(NodeKey, Span)>,
}
#[derive(Clone, Copy)]
struct Ordered {
    entry: Entry,
    ordinal: usize,
}
impl Draft {
    pub(super) fn error(&self, message: &'static str) -> Error {
        self.error_site.map_or_else(
            || Error::unlocated(message),
            |(owner, span)| Error::at(owner.file, span.start, message),
        )
    }
    pub(super) fn steps(&self) -> usize {
        self.budget.work.steps
    }
    pub(super) fn collect(
        &mut self,
        package: &Package,
        facts: &Resolution,
        owner: NodeKey,
        attribute: Attribute,
        forest: &Forest,
    ) -> Result<Option<DiagnosticItem>, Error> {
        let error = |message| Error::at(owner.file, attribute.span.start, message);
        self.budget
            .pay(Growth {
                work: attribute.span.end.saturating_sub(attribute.span.start) as usize + 1,
                bytes: (attribute.span.end.saturating_sub(attribute.span.start) as usize)
                    .saturating_mul(4),
            })
            .map_err(error)?;
        if attribute.unsafe_wrapper {
            return Err(error("diagnostic identity cannot use an unsafe wrapper"));
        }
        if !package
            .source_core()
            .is_some_and(|s| s.features.has(Feature::RustcAttributes))
        {
            return Err(error(
                "diagnostic identity requires its rustc_attrs source provider",
            ));
        }
        let unit = package.unit(owner.file);
        let bytes = decode(unit, owner.file, attribute, forest)?;
        if let Some(known) = annotations::known(&bytes) {
            return Ok(Some(known));
        }
        if !self.target(package, facts, owner).map_err(error)? {
            return Err(error(
                "diagnostic item has an unsupported declaration target",
            ));
        }
        let name =
            std::str::from_utf8(&bytes).map_err(|_| error("diagnostic string requires UTF-8"))?;
        self.budget.names(name.len()).map_err(error)?;
        let mut growth = Growth::vector::<Pending>(self.pending.len(), self.pending.capacity(), 1);
        growth.bytes = growth.bytes.saturating_add(name.len());
        self.budget.pay(growth).map_err(error)?;
        self.pending.push(Pending {
            name: name.into(),
            owner,
            attribute: attribute.span,
        });
        Ok(None)
    }
    fn target(
        &mut self,
        package: &Package,
        facts: &Resolution,
        owner: NodeKey,
    ) -> Result<bool, &'static str> {
        let ast = &package.unit(owner.file).configured.ast;
        let Kind::Item(index) = ast.node(owner.node).kind else {
            return Ok(matches!(ast.node(owner.node).kind, Kind::Crate(_)));
        };
        Ok(match ast.items[index as usize].kind {
            ItemKind::Trait { .. }
            | ItemKind::Struct { .. }
            | ItemKind::Enum { .. }
            | ItemKind::MacroRules { .. }
            | ItemKind::DeclarativeMacro { .. }
            | ItemKind::TypeAlias { .. }
            | ItemKind::Const { .. }
            | ItemKind::Module { .. }
            | ItemKind::Impl {
                trait_path: None, ..
            }
            | ItemKind::Function { body: Some(_), .. } => true,
            ItemKind::Function { body: None, .. } => !self.is_foreign(facts, owner)?,
            _ => false,
        })
    }
    fn is_foreign(&mut self, facts: &Resolution, owner: NodeKey) -> Result<bool, &'static str> {
        if self.foreign.is_none() {
            let count = facts.foreign_owners.len();
            self.budget.pay(Growth {
                work: count.saturating_add(1),
                bytes: count.saturating_mul(size_of::<NodeKey>()),
            })?;
            let mut nodes = Vec::with_capacity(count);
            for definition in facts.foreign_owners.keys() {
                nodes.push(
                    facts
                        .definitions
                        .get(definition.0 as usize)
                        .and_then(|d| d.node)
                        .ok_or("foreign diagnostic target lost its actual declaration")?,
                );
            }
            self.sort(&mut nodes, |a, b| key(*a).cmp(&key(*b)))?;
            self.foreign = Some(nodes);
        }
        let nodes = self
            .foreign
            .as_deref()
            .ok_or("foreign diagnostic target index missing")?;
        self.budget.pay(Growth {
            work: usize::try_from(nodes.len().checked_ilog2().unwrap_or(0))
                .unwrap_or(usize::MAX)
                .saturating_add(2),
            bytes: 0,
        })?;
        Ok(nodes
            .binary_search_by_key(&key(owner), |node| key(*node))
            .is_ok())
    }
    fn sort<T>(
        &mut self,
        values: &mut [T],
        mut compare: impl FnMut(&T, &T) -> Ordering,
    ) -> Result<(), &'static str> {
        self.budget.pay(Growth {
            work: 1,
            bytes: values.len().saturating_mul(2 * size_of::<usize>()),
        })?;
        let result = canonical_order(values, &mut self.budget.work, &mut 0, |a, b, work| {
            work.charge(1)?;
            Ok(compare(a, b))
        });
        if result.is_err() {
            self.budget.failed = true;
        }
        result
    }
    pub(super) fn seal(&mut self, names: &mut Names) -> Result<Option<Box<Column>>, &'static str> {
        let result = self.seal_inner(names);
        if result.is_err() {
            self.budget.failed = true;
        }
        result
    }
    fn seal_inner(&mut self, names: &mut Names) -> Result<Option<Box<Column>>, &'static str> {
        self.budget.pay(Growth { work: 1, bytes: 0 })?;
        let count = self.pending.len();
        if count == 0 {
            return Ok(None);
        }
        self.budget.pay(Growth {
            work: count,
            bytes: count.saturating_mul(size_of::<Ordered>()),
        })?;
        let mut rows = Vec::with_capacity(count);
        for (ordinal, pending) in self.pending.iter().enumerate() {
            self.error_site = Some((pending.owner, pending.attribute));
            let name = names.intern_exact_paid(&pending.name, &mut |cost| self.budget.pay(cost))?;
            rows.push(Ordered {
                entry: Entry {
                    name,
                    owner: pending.owner,
                    attribute: pending.attribute,
                },
                ordinal,
            });
        }
        self.sort(&mut rows, |a, b| {
            (a.entry.name.0, a.ordinal).cmp(&(b.entry.name.0, b.ordinal))
        })?;
        self.budget.pay(Growth {
            work: count,
            bytes: 0,
        })?;
        if let Some(duplicate) = rows
            .windows(2)
            .filter(|pair| pair[0].entry.name == pair[1].entry.name)
            .map(|pair| pair[1])
            .min_by_key(|row| row.ordinal)
        {
            self.error_site = Some((duplicate.entry.owner, duplicate.entry.attribute));
            return Err("duplicate source diagnostic identity");
        }
        self.sort(&mut rows, |a, b| {
            (key(a.entry.owner), a.entry.attribute.start)
                .cmp(&(key(b.entry.owner), b.entry.attribute.start))
        })?;
        self.budget.pay(Growth {
            work: count,
            bytes: size_of::<Column>().saturating_add(count.saturating_mul(size_of::<Entry>())),
        })?;
        let rows = rows.into_iter().map(|row| row.entry).collect();
        self.pending.clear();
        let column = Column { rows };
        if retained::vector(&column.rows).saturating_add(size_of::<Column>()) > self.budget.bytes {
            return Err("diagnostic capacity exceeded its prepaid storage");
        }
        Ok(Some(Box::new(column)))
    }
}
fn key(node: NodeKey) -> (u32, u32) {
    (node.file.0, node.node.get())
}
fn decode<'a>(
    unit: &'a Unit,
    file: FileId,
    attribute: Attribute,
    forest: &Forest,
) -> Result<std::borrow::Cow<'a, [u8]>, Error> {
    let error = |message| Error::at(file, attribute.span.start, message);
    let Input::Equals(range) = attribute.input else {
        return Err(error("diagnostic item requires one string literal"));
    };
    if range.end != range.start + 1 {
        return Err(error("diagnostic item requires one string literal"));
    }
    let token = forest
        .token(range.start)
        .ok_or_else(|| error("missing diagnostic item literal"))?;
    let Some(crate::literal::Value::String(bytes)) =
        crate::literal::interpret(unit.source.text(), token)
            .map_err(|e| Error::at(file, e.offset, e.message))?
    else {
        return Err(error("diagnostic item requires one string literal"));
    };
    Ok(bytes)
}
#[cfg(test)]
#[path = "../../../tests/invariants/inert_diagnostics.rs"]
mod tests;
