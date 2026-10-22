//! Deterministic inspection of proved contracts, without loan or executable permission.
use super::{Analysis, graph::Kind};
use std::io::{self, Write};

/// # Errors
/// Propagates output failures or malformed retained graph identities.
pub fn write(analysis: &Analysis, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "regions-crate 1")?;
    let mut variables = Vec::new();
    let mut parameters = Vec::new();
    let mut assumptions = Vec::new();
    let mut requirements = Vec::new();
    for (index, body) in analysis.bodies.iter().enumerate() {
        writeln!(
            out,
            "body {index} source={}:{}",
            body.source.file.0,
            body.source.node.get()
        )?;
        for (id, region) in body.identities.universals().enumerate() {
            writeln!(out, "  universal b{id}=g{}", region.0)?;
        }
        parameters.clear();
        parameters.extend(body.identities.parameters());
        parameters.sort_unstable_by_key(|&(id, _)| id.0);
        for &(id, region) in &parameters {
            writeln!(out, "  type-parameter p{}=g{}", id.0, region.0)?;
        }
        variables.clear();
        variables.extend(body.identities.variables());
        variables.sort_unstable_by_key(|&(id, _)| id.0);
        for &(id, region) in &variables {
            writeln!(out, "  variable r{}=g{}", id.0, region.0)?;
        }
        assumptions.clear();
        requirements.clear();
        for (id, kind) in body.solution.nodes() {
            write!(out, "  region g{} ", id.0)?;
            match kind {
                Kind::Static => writeln!(out, "static")?,
                Kind::Local => writeln!(out, "local")?,
                Kind::Universal { universe } => writeln!(out, "universal universe={universe}")?,
                Kind::Variable { universe } => writeln!(out, "variable universe={universe}")?,
            }
            assumptions.extend(
                body.solution
                    .assumptions(id)
                    .map_err(io::Error::other)?
                    .map(|shorter| (id.0, shorter.0)),
            );
            requirements.extend(
                body.solution
                    .requirements(id)
                    .map_err(io::Error::other)?
                    .map(|requirement| (requirement.longer.0, id.0)),
            );
        }
        assumptions.sort_unstable();
        requirements.sort_unstable();
        for &(longer, shorter) in &assumptions {
            writeln!(out, "  assume g{longer}:g{shorter}")?;
        }
        for &(longer, shorter) in &requirements {
            writeln!(out, "  require g{longer}:g{shorter}")?;
        }
        writeln!(out, "end-body")?;
    }
    write!(out, "eof ")?;
    count(analysis, out)
}

/// Count the same retained facts without sorting or materializing output records.
/// # Errors
/// Propagates output failures.
pub fn count(analysis: &Analysis, out: &mut impl Write) -> io::Result<()> {
    let mut universals = 0;
    let mut variables = 0;
    let mut regions = 0;
    let mut assumptions = 0;
    let mut requirements = 0;
    for body in &analysis.bodies {
        universals += body.identities.universals().len();
        variables += body.identities.variables().len();
        regions += body.solution.regions;
        assumptions += body.solution.assumption_count();
        requirements += body.solution.obligations;
    }
    writeln!(
        out,
        "bodies={} universals={universals} variables={variables} regions={regions} assumptions={assumptions} requirements={requirements}",
        analysis.bodies.len()
    )
}
