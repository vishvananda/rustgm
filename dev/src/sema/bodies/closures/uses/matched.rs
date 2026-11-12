//! A pattern matched against a captured place uses it as rustc's capture
//! analysis walks the pattern.
//!
//! A variant, literal, range or slice-length test reads the place. A binding
//! copies, moves or borrows it. Precision stops at an enum variant and at an
//! index projection, so those uses capture the matched place itself. A field
//! projection or a dereference before that point would capture a more precise
//! place, which fails closed. Whole-variable editions and moving closures
//! capture the variable or its reference instead.
use super::{Checker, Collector, Error, NodeId, Use, Work};
use crate::sema::patterns::{Capture, Kind, PatternId, Projection};

#[derive(Clone, Copy)]
struct Site {
    pattern: PatternId,
    /// Dereferences applied to the matched place before any projection.
    dereferences: u32,
    /// A projection fixes the leading dereference count.
    projected: bool,
    /// A variant or index projection captures the place reached so far.
    truncated: bool,
    /// A field projection or dereference preceded truncation.
    precise: bool,
}
impl Site {
    const fn new(pattern: PatternId) -> Self {
        Self {
            pattern,
            dereferences: 0,
            projected: false,
            truncated: false,
            precise: false,
        }
    }
    const fn child(self, pattern: PatternId) -> Self {
        Self { pattern, ..self }
    }
    const fn truncate(self, pattern: PatternId) -> Self {
        Self {
            pattern,
            projected: true,
            truncated: true,
            ..self
        }
    }
}
impl Collector<'_> {
    /// Records the scrutinee uses of every root pattern matched against the
    /// captured place `scrutinee`; a place that no pattern uses is mentioned.
    pub(super) fn pattern_uses(
        &mut self,
        checker: &Checker<'_>,
        scrutinee: NodeId,
        roots: &[PatternId],
    ) -> Result<(), Error> {
        let whole = self.places.graph.captures_whole_variables();
        let mut sites: Vec<_> = roots.iter().rev().copied().map(Site::new).collect();
        let mut used = false;
        while let Some(site) = sites.pop() {
            self.work += 1;
            if self.work > 1_000_000 {
                return Err(checker.error(scrutinee, "closure use work limit exceeded"));
            }
            let patterns = &checker.facts.patterns;
            let kind = patterns
                .nodes
                .get(site.pattern.0 as usize)
                .ok_or_else(|| checker.error(scrutinee, "missing capture pattern"))?
                .kind;
            let usage = match kind {
                Kind::Pending => {
                    return Err(checker.error(scrutinee, "capture pattern is not selected"));
                }
                Kind::Range(_) | Kind::Text(_) | Kind::Slice { .. } => Some(Use::Shared),
                Kind::Product {
                    variant: Some(variant),
                    ..
                } if checker.facts.members.needs_test(variant) => Some(Use::Shared),
                Kind::Binding {
                    capture: Capture::Borrow { mutable, .. },
                    ..
                } => Some(if mutable { Use::Mutable } else { Use::Shared }),
                Kind::Binding {
                    definition,
                    capture: Capture::Value,
                    ..
                } => Some(Use::ValueOf(
                    checker.state.locals[definition.0 as usize]
                        .ok_or_else(|| checker.error(scrutinee, "missing capture binding type"))?,
                )),
                Kind::Wildcard | Kind::Product { .. } | Kind::Dereference { .. } | Kind::Or(_) => {
                    None
                }
            };
            match kind {
                Kind::Binding {
                    subpattern: Some(pattern),
                    ..
                } => sites.push(site.child(pattern)),
                Kind::Dereference { pattern, .. } => {
                    sites.push(self.dereferenced(site.child(pattern), whole));
                }
                Kind::Or(alternatives) => sites.extend(
                    patterns
                        .alternatives(alternatives)
                        .iter()
                        .rev()
                        .map(|&pattern| site.child(pattern)),
                ),
                Kind::Product { variant, fields } => {
                    let downcast = variant.is_some_and(|v| checker.facts.members.needs_test(v));
                    sites.extend(patterns.fields(fields).iter().rev().map(|field| {
                        match field.selection {
                            Projection::Field(_) if !downcast => Site {
                                projected: true,
                                precise: site.precise || !site.truncated,
                                ..site.child(field.pattern)
                            },
                            _ => site.truncate(field.pattern),
                        }
                    }));
                }
                Kind::Slice { fields, .. } => sites.extend(
                    patterns
                        .fields(fields)
                        .iter()
                        .rev()
                        .map(|field| site.truncate(field.pattern)),
                ),
                _ => {}
            }
            let Some(usage) = usage else {
                continue;
            };
            if site.precise && !whole {
                return Err(
                    checker.error(scrutinee, "precise capture patterns are not implemented")
                );
            }
            used = true;
            self.pending.push(Work {
                node: scrutinee,
                usage,
                dereferences: site.dereferences,
                receiver: None,
            });
        }
        if !used {
            self.push(scrutinee, Use::Mention);
        }
        Ok(())
    }
    /// A whole-variable or moving capture keeps the matched variable, so only
    /// leading dereferences are applied to it. A moving closure captures the
    /// reference itself, which truncates the place.
    const fn dereferenced(&self, site: Site, whole: bool) -> Site {
        if site.truncated {
            return site;
        }
        if !whole && !self.moving {
            return Site {
                precise: true,
                ..site
            };
        }
        let dereferences = if site.projected {
            site.dereferences
        } else {
            site.dereferences.saturating_add(1)
        };
        Site {
            dereferences,
            projected: site.projected || !whole,
            truncated: !whole,
            ..site
        }
    }
}
