//! Strings are partitioned by observed values plus one class for every other string.
use super::{Coverage, Kind, Roots};
use crate::sema::text::Id;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct Domain {
    pub values: Vec<Id>,
    pub ranks: HashMap<Id, u128>,
}
impl Coverage<'_> {
    pub(super) fn prepare_texts(&mut self) -> Result<(), &'static str> {
        if self.texts.is_some() {
            return Ok(());
        }
        let mut domain = Domain::default();
        let mut seen = HashSet::new();
        let mut work = match self.roots {
            Roots::Pattern(id) => vec![id],
            Roots::Arms(arms) => arms.iter().filter_map(|arm| arm.pattern).collect(),
        };
        while let Some(id) = work.pop() {
            self.graph.charge()?;
            if !seen.insert(id) {
                continue;
            }
            match self.facts.patterns.nodes[id.0 as usize].kind {
                Kind::Text(id) => {
                    if let std::collections::hash_map::Entry::Vacant(entry) = domain.ranks.entry(id)
                    {
                        entry.insert(domain.values.len() as u128);
                        domain.values.push(id);
                    }
                }
                Kind::Product { fields, .. } | Kind::Slice { fields, .. } => {
                    for field in self.facts.patterns.fields(fields) {
                        self.graph.charge()?;
                        work.push(field.pattern);
                    }
                }
                Kind::Binding { subpattern, .. } => work.extend(subpattern),
                Kind::Dereference { pattern, .. } => work.push(pattern),
                Kind::Or(alternatives) => {
                    for &pattern in self.facts.patterns.alternatives(alternatives) {
                        self.graph.charge()?;
                        work.push(pattern);
                    }
                }
                Kind::Pending => return Err("incomplete text coverage pattern"),
                Kind::Wildcard | Kind::Range(_) => {}
            }
        }
        self.texts = Some(domain);
        Ok(())
    }
}
