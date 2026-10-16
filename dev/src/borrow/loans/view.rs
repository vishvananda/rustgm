//! Render checked loans and sparse states; inspection is never semantic transport.
use super::{Mode, inspection::Detail};
use crate::{
    body::ir::{BlockId, Body},
    borrow::Analysis,
};
use std::io::{self, Write};

fn validate(analysis: &Analysis<'_>) -> io::Result<()> {
    let program = analysis.checked().program();
    if analysis.inspection.len() != program.bodies.len()
        || analysis
            .inspection
            .iter()
            .zip(&program.bodies)
            .any(|(a, b)| a.source != b.source)
    {
        return Err(io::Error::other(
            "loan view requires retained inspection facts",
        ));
    }
    Ok(())
}
/// # Errors
/// Rejects missing inspection facts before output; propagates output failures.
pub fn write(analysis: &Analysis<'_>, out: &mut impl Write) -> io::Result<()> {
    validate(analysis)?;
    writeln!(out, "loans-crate 1")?;
    for (index, (inspection, body)) in analysis
        .inspection
        .iter()
        .zip(&analysis.checked().program().bodies)
        .enumerate()
    {
        writeln!(
            out,
            "body {index} source={}:{}",
            body.source.file.0, body.source.node.0
        )?;
        if let Some(detail) = &inspection.detail {
            write_body(body, detail, out)?;
        }
        writeln!(out, "end-body")?;
    }
    write!(out, "eof ")?;
    count(analysis, out)
}
fn write_body(body: &Body, detail: &Detail, out: &mut impl Write) -> io::Result<()> {
    detail.places.write(out)?;
    for (id, loan) in detail.loans.iter().enumerate() {
        let kind = match loan.mode {
            Mode::Shared => "shared",
            Mode::Mutable => "mutable",
            Mode::Reserved => "two-phase",
        };
        writeln!(
            out,
            "  loan q{id} borrow=v{} place=p{} region=g{} kind={kind}",
            loan.value.0, loan.place.0, loan.region.0
        )?;
    }
    for (id, block) in (0_u32..).zip(&body.blocks) {
        let reachable = detail.uses.reachable[id as usize];
        for at in block.start..=block.end {
            let point = detail
                .uses
                .point_before(BlockId(id), at)
                .ok_or_else(|| io::Error::other("missing inspected loan point"))?;
            write!(out, "  point t{} block=b{id} at=", point.0)?;
            if at == block.end {
                write!(out, "end")?;
            } else {
                write!(out, "s{at}")?;
            }
            writeln!(out, " reachable={reachable}")?;
        }
    }
    for (region, word, bits) in detail.regions.words() {
        writeln!(
            out,
            "  live-region g{} word={word} bits={bits:016x}",
            region.0
        )?;
    }
    for snapshot in &detail.snapshots {
        writeln!(
            out,
            "  state t{} word={} live={:016x} activated={:016x}",
            snapshot.point.0, snapshot.word, snapshot.live, snapshot.active
        )?;
    }
    Ok(())
}
/// # Errors
/// Rejects missing inspection facts before output; propagates output failures.
pub fn count(analysis: &Analysis<'_>, out: &mut impl Write) -> io::Result<()> {
    validate(analysis)?;
    let mut loans = 0;
    let mut places = 0;
    let mut points = 0;
    let mut region_words = 0;
    let mut state_words = 0;
    for detail in analysis.inspection.iter().filter_map(|i| i.detail.as_ref()) {
        loans += detail.loans.len();
        places += detail.places.count();
        points += u64::from(detail.uses.points);
        region_words += detail.regions.mask_count();
        state_words += detail.snapshots.len();
    }
    writeln!(
        out,
        "bodies={} loans={loans} places={places} points={points} region_words={region_words} state_words={state_words}",
        analysis.inspection.len()
    )
}
