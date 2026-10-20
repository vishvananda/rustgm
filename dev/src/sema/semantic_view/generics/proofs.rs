//! Borrow retained scope and relation records; serialization never performs proof.
use super::{Declarations, io, type_view};
use crate::types::lifetime_proof::{Obligations, ScopeId};
use std::io::Write;

pub(super) fn write(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    for (id, value) in facts.quantified_proofs.iter().enumerate() {
        let evidence = &value.evidence;
        writeln!(
            out,
            "proof {id} body={}:{} source={}:{} pending",
            value.body.file.0, value.body.node.0, evidence.source.file.0, evidence.source.node.0
        )?;
        for (index, source) in evidence.sources.iter().enumerate() {
            writeln!(
                out,
                "proof-source proof={id} index={index} source={}:{}",
                source.file.0, source.node.0
            )?;
        }
        let proof = &evidence.proof;
        let plan = proof.scope_plan();
        for index in 1..=plan.len() {
            let scope = ScopeId(u32::try_from(index).map_err(io::Error::other)?);
            let (parent, count) = plan.binder(scope).map_err(io::Error::other)?;
            writeln!(
                out,
                "proof-scope proof={id} scope={index} parent={} regions={count} depth={}",
                parent.0,
                plan.depth(scope).map_err(io::Error::other)?
            )?;
        }
        for range in proof.owned_regions() {
            writeln!(
                out,
                "proof-owned proof={id} start={} end={}",
                range.start, range.end
            )?;
        }
        for (index, (scope, aliases, source, block)) in proof.blocks().enumerate() {
            writeln!(
                out,
                "proof-block proof={id} index={index} scope={} aliases={aliases} source={source} start={} end={}",
                scope.0, block.regions.start, block.regions.end
            )?;
            relations(id, index, block, out)?;
        }
    }
    Ok(())
}

fn relations(id: usize, block: usize, proof: &Obligations, out: &mut impl Write) -> io::Result<()> {
    for (kind, pairs) in [("equal", &proof.equalities), ("outlives", &proof.outlives)] {
        for &(a, b) in pairs {
            write!(
                out,
                "proof-relation proof={id} block={block} kind={kind} left="
            )?;
            type_view::region(a, out)?;
            write!(out, " right=")?;
            type_view::region(b, out)?;
            writeln!(out)?;
        }
    }
    for &(ty, shorter) in &proof.types {
        write!(
            out,
            "proof-type proof={id} block={block} type={} shorter=",
            ty.0
        )?;
        type_view::region(shorter, out)?;
        writeln!(out)?;
    }
    Ok(())
}

pub(super) fn count(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    let mut scopes = 0;
    let mut blocks = 0;
    let mut sources = 0;
    let mut intervals = 0;
    let mut relations = 0;
    let mut types = 0;
    for value in &facts.quantified_proofs {
        let proof = &value.evidence.proof;
        scopes += proof.scope_count();
        sources += value.evidence.sources.len();
        intervals += proof.owned_regions().count();
        for (_, _, _, block) in proof.blocks() {
            blocks += 1;
            relations += block.equalities.len() + block.outlives.len();
            types += block.types.len();
        }
    }
    write!(
        out,
        " proofs={} proof-scopes={scopes} proof-blocks={blocks} proof-sources={sources} proof-owned={intervals} proof-relations={relations} proof-types={types}",
        facts.quantified_proofs.len()
    )
}
