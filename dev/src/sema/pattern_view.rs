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
    for (ordinal, field) in facts.patterns.fields.iter().enumerate() {
        write!(out, "field {ordinal} ")?;
        match field.selection {
            Projection::Field(Selection::Tuple(index)) => write!(out, "tuple={index}")?,
            Projection::Field(Selection::Nominal(field)) => write!(out, "nominal={}", field.0)?,
            Projection::Index(index) => write!(out, "index={index}")?,
            Projection::Subarray { start, end } => write!(out, "subarray={start}:{end}")?,
            Projection::Subslice { prefix, suffix } => write!(out, "subslice={prefix}:{suffix}")?,
        }
        writeln!(out, " pattern={}", field.pattern.0)?;
    }
    for (ordinal, alternative) in facts.patterns.alternatives.iter().enumerate() {
        writeln!(out, "alternative {ordinal} pattern={}", alternative.0)?;
    }
    type_view::body_regions(&facts.body_regions, out)?;
    type_view::regions(&facts.region_equalities, out)?;
    write!(out, "eof ")?;
    count(facts, out)
}
/// # Errors
/// Propagates output failures after the same analysis as the complete view.
pub fn count(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    writeln!(
        out,
        "roots={} patterns={} fields={} alternatives={} region-vars={} region-equalities={}",
        facts.patterns.roots.len(),
        facts.patterns.nodes.len(),
        facts.patterns.fields.len(),
        facts.patterns.alternatives.len(),
        facts.body_regions.len(),
        facts.region_equalities.len()
    )
}
