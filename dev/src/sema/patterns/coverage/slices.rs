//! Finite length partitions preserve prefix/suffix aliasing without enumerating values.
use super::{Coverage, Id, Kind, PatternId, Query, Roots, Step, Work};
use crate::sema::patterns::{Fields, Projection};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct Positions {
    prefix: HashSet<u64>,
    suffix: HashSet<u64>,
}
pub(super) struct Branch {
    pub length: Id,
    pub children: Vec<Query>,
}

impl Coverage<'_> {
    /// Collect tested positions once from this query, then split only at possible aliases.
    fn prepare_slices(&mut self) -> Result<(), &'static str> {
        if self.slice_aliases.is_some() {
            return Ok(());
        }
        let mut bounds = HashMap::new();
        let mut seen = HashSet::new();
        let mut work = match self.roots {
            Roots::Pattern(pattern) => vec![pattern],
            Roots::Arms(arms) => arms.iter().filter_map(|arm| arm.pattern).collect(),
        };
        while let Some(id) = work.pop() {
            self.graph.charge()?;
            if !seen.insert(id) {
                continue;
            }
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            match pattern.kind {
                Kind::Slice { fields, .. } => {
                    let ty = pattern.ty.ok_or("unsolved slice coverage type")?;
                    let entry: &mut Positions = bounds.entry(ty).or_default();
                    for field in self.facts.patterns.fields(fields) {
                        self.graph.charge()?;
                        work.push(field.pattern);
                        if !self.slice_tested(field.pattern)? {
                            continue;
                        }
                        match field.selection {
                            Projection::Index(index) => {
                                entry.prefix.insert(index);
                            }
                            Projection::FromEnd(offset) => {
                                entry.suffix.insert(offset);
                            }
                            Projection::Subslice { .. } => {}
                            _ => return Err("invalid slice coverage field"),
                        }
                    }
                }
                Kind::Product { fields, .. } => {
                    work.extend(self.facts.patterns.fields(fields).iter().map(|f| f.pattern));
                }
                Kind::Binding { subpattern, .. } => work.extend(subpattern),
                Kind::Dereference { pattern, .. } => work.push(pattern),
                Kind::Or(alternatives) => {
                    work.extend(self.facts.patterns.alternatives(alternatives));
                }
                Kind::Pending => return Err("incomplete slice coverage pattern"),
                Kind::Wildcard | Kind::Range(_) | Kind::Text(_) => {}
            }
        }
        let mut aliases = HashMap::new();
        for (ty, positions) in bounds {
            let mut lengths = HashSet::new();
            for start in positions.prefix {
                for &end in &positions.suffix {
                    self.graph.charge()?;
                    lengths.insert(
                        start
                            .checked_add(end)
                            .ok_or("slice alias length overflow")?,
                    );
                }
            }
            let mut lengths: Vec<_> = lengths.into_iter().collect();
            lengths.sort_unstable();
            aliases.insert(ty, lengths);
        }
        self.slice_aliases = Some(aliases);
        Ok(())
    }

    fn slice_tested(&mut self, mut id: PatternId) -> Result<bool, &'static str> {
        loop {
            self.graph.charge()?;
            match self.facts.patterns.nodes[id.0 as usize].kind {
                Kind::Wildcard
                | Kind::Binding {
                    subpattern: None, ..
                } => return Ok(false),
                Kind::Binding {
                    subpattern: Some(child),
                    ..
                } => id = child,
                _ => return Ok(true),
            }
        }
    }

    pub(super) fn slice(
        &mut self,
        query: Query,
        minimum: u64,
        exact: bool,
        fields: Fields,
    ) -> Result<Work, &'static str> {
        let has_suffix = self
            .facts
            .patterns
            .fields(fields)
            .iter()
            .any(|field| matches!(field.selection, Projection::FromEnd(_)));
        let mut branches = Vec::new();
        let mut lower = minimum;
        if !exact && has_suffix {
            self.prepare_slices()?;
            let ty = self.places[query.place.0 as usize].ty;
            let lengths = self
                .slice_aliases
                .as_ref()
                .and_then(|b| b.get(&ty))
                .ok_or("missing slice coverage aliases")?;
            let mut index = lengths.partition_point(|n| *n < minimum);
            while let Some(length) = self
                .slice_aliases
                .as_ref()
                .and_then(|b| b.get(&ty))
                .and_then(|lengths| lengths.get(index))
                .copied()
            {
                self.graph.charge()?;
                if lower < length {
                    branches.push(self.slice_branch(query, fields, (lower, length - 1), false)?);
                }
                branches.push(self.slice_branch(query, fields, (length, length), true)?);
                lower = length
                    .checked_add(1)
                    .ok_or("slice length successor overflow")?;
                index += 1;
            }
        }
        branches.push(self.slice_branch(
            query,
            fields,
            (lower, if exact { minimum } else { u64::MAX }),
            false,
        )?);
        Ok(Work::Slices { query, branches })
    }

    fn slice_branch(
        &mut self,
        query: Query,
        fields: Fields,
        (lower, upper): (u64, u64),
        canonical: bool,
    ) -> Result<Branch, &'static str> {
        self.graph.charge()?;
        let variable = self.variable(query.place)?;
        let length = self
            .graph
            .interval(variable, u128::from(lower), u128::from(upper))?;
        let mut children = Vec::new();
        for field in self.facts.patterns.fields(fields) {
            self.graph.charge()?;
            let selection = match field.selection {
                Projection::FromEnd(offset) if canonical => Projection::Index(
                    lower
                        .checked_sub(offset)
                        .ok_or("invalid slice suffix offset")?,
                ),
                selection => selection,
            };
            children.push(self.child(query.place, field.pattern, Step::Field(selection))?);
        }
        Ok(Branch { length, children })
    }
}
