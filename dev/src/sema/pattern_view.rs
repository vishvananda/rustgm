//! Sparse selected patterns expose capture operations before region and loan proofs.
use super::{
    members::Selection,
    model::Declarations,
    patterns::{Capture, Kind, Projection},
    type_view,
};
use std::io::{self, Write};

/// # Errors
/// Propagates output failures and rejects incomplete selected facts.
pub fn write(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "selected-patterns 1")?;
    type_view::types(&facts.types, out)?;
    let mut roots: Vec<_> = facts.patterns.roots.iter().collect();
    roots.sort_unstable_by_key(|(source, _)| (source.file.0, source.node.0));
    for (source, id) in roots {
        writeln!(
            out,
            "root {}:{} pattern={}",
            source.file.0, source.node.0, id.0
        )?;
    }
    nodes(facts, out)?;
    for (ordinal, field) in facts.patterns.fields.iter().enumerate() {
        write!(out, "field {ordinal} ")?;
        match field.selection {
            Projection::Field(Selection::Tuple(index)) => write!(out, "tuple={index}")?,
            Projection::Field(Selection::Nominal(field)) => write!(out, "nominal={}", field.0)?,
            Projection::Index(index) => write!(out, "index={index}")?,
            Projection::FromEnd(offset) => write!(out, "from-end={offset}")?,
            Projection::Subarray { start, end } => write!(out, "subarray={start}:{end}")?,
            Projection::Subslice { prefix, suffix } => write!(out, "subslice={prefix}:{suffix}")?,
        }
        writeln!(out, " pattern={}", field.pattern.0)?;
    }
    for (ordinal, alternative) in facts.patterns.alternatives.iter().enumerate() {
        writeln!(out, "alternative {ordinal} pattern={}", alternative.0)?;
    }
    for (ordinal, interval) in facts.patterns.ranges.iter().enumerate() {
        writeln!(
            out,
            "range {ordinal} ranks={}:{}",
            interval.lower, interval.upper
        )?;
    }
    for (ordinal, decision) in facts.patterns.decisions.iter().enumerate() {
        writeln!(
            out,
            "match {ordinal} source={}:{} input={}:{} arms={}:{} exhaustive=true uninhabited={} work={}",
            decision.source.file.0,
            decision.source.node.0,
            decision.scrutinee.file.0,
            decision.scrutinee.node.0,
            decision.start,
            decision.end,
            decision.uninhabited,
            decision.work
        )?;
    }
    for (ordinal, arm) in facts.patterns.arms.iter().enumerate() {
        let pattern = arm
            .pattern
            .ok_or_else(|| io::Error::other("missing match arm selection"))?;
        write!(
            out,
            "arm {ordinal} source={}:{} pattern={} guard=",
            arm.source.file.0, arm.source.node.0, pattern.0
        )?;
        if let Some(guard) = arm.guard {
            write!(out, "{}:{}", guard.file.0, guard.node.0)?;
        } else {
            write!(out, "-")?;
        }
        writeln!(
            out,
            " useful={} value={}:{}",
            arm.useful, arm.value.file.0, arm.value.node.0
        )?;
    }
    let mut conditions: Vec<_> = facts.patterns.conditions.iter().collect();
    conditions.sort_unstable_by_key(|(source, _)| (source.file.0, source.node.0));
    for (source, condition) in conditions {
        write!(
            out,
            "condition source={}:{} input={}:{} pattern={} otherwise=",
            source.file.0,
            source.node.0,
            condition.input.file.0,
            condition.input.node.0,
            condition.pattern.0
        )?;
        if let Some(otherwise) = condition.otherwise {
            writeln!(out, "{}:{}", otherwise.file.0, otherwise.node.0)?;
        } else {
            writeln!(out, "-")?;
        }
    }
    type_view::body_regions(&facts.body_regions, out)?;
    type_view::regions(&facts.region_equalities, out)?;
    type_view::coercions(&facts.coercions, out)?;
    type_view::outlives(&facts.outlives, out)?;
    write!(out, "eof ")?;
    count(facts, out)
}
fn nodes(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    for (ordinal, pattern) in facts.patterns.nodes.iter().enumerate() {
        let ty = pattern
            .ty
            .ok_or_else(|| io::Error::other("unsolved pattern type"))?;
        write!(
            out,
            "pattern {ordinal} source={}:{} input={} ",
            pattern.source.file.0, pattern.source.node.0, ty.0
        )?;
        match pattern.kind {
            Kind::Pending => return Err(io::Error::other("incomplete selected pattern")),
            Kind::Wildcard => write!(out, "wildcard")?,
            Kind::Range(id) => write!(out, "range={}", id.0)?,
            Kind::Text(id) => {
                write!(out, "text=")?;
                crate::support::quoted::bytes(
                    out,
                    facts.texts.get(id).map_err(io::Error::other)?,
                    false,
                )?;
            }
            Kind::Binding {
                definition,
                mutable,
                capture,
                subpattern,
            } => {
                let local = facts.definitions[definition.0 as usize]
                    .ok_or_else(|| io::Error::other("unsolved pattern binding type"))?;
                write!(
                    out,
                    "bind def={} local={} mutable={mutable} capture=",
                    definition.0, local.0
                )?;
                match capture {
                    Capture::Value => write!(out, "value")?,
                    Capture::Borrow { mutable, region } => {
                        write!(out, "{} region=", if mutable { "ref-mut" } else { "ref" })?;
                        type_view::region(region, out)?;
                        write!(out, " loan=pending")?;
                    }
                }
                write!(out, " subpattern=")?;
                if let Some(child) = subpattern {
                    write!(out, "{}", child.0)?;
                } else {
                    write!(out, "-")?;
                }
            }
            Kind::Dereference {
                mutable,
                implicit,
                pattern,
            } => {
                write!(
                    out,
                    "deref mutable={mutable} implicit={implicit} pattern={}",
                    pattern.0
                )?;
            }
            Kind::Product { variant, fields } => {
                write!(out, "product variant=")?;
                if let Some(variant) = variant {
                    write!(out, "{}", variant.0)?;
                } else {
                    write!(out, "-")?;
                }
                write!(out, " fields={}:{}", fields.start, fields.end)?;
            }
            Kind::Slice {
                minimum,
                exact,
                fields,
            } => {
                write!(
                    out,
                    "slice minimum={minimum} exact={exact} fields={}:{}",
                    fields.start, fields.end
                )?;
            }
            Kind::Or(alternatives) => {
                write!(
                    out,
                    "or alternatives={}:{}",
                    alternatives.start, alternatives.end
                )?;
            }
        }
        writeln!(out)?;
    }
    Ok(())
}
/// # Errors
/// Propagates output failures after the same analysis as the complete view.
pub fn count(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    writeln!(
        out,
        "roots={} patterns={} fields={} alternatives={} ranges={} matches={} arms={} coverage-work={} region-vars={} region-equalities={} conditions={} chains={}",
        facts.patterns.roots.len(),
        facts.patterns.nodes.len(),
        facts.patterns.fields.len(),
        facts.patterns.alternatives.len(),
        facts.patterns.ranges.len(),
        facts.patterns.decisions.len(),
        facts.patterns.arms.len(),
        facts.patterns.decisions.iter().map(|m| m.work).sum::<u64>(),
        facts.body_regions.len(),
        facts.region_equalities.len(),
        facts.patterns.conditions.len(),
        facts.patterns.chains.len()
    )
}
