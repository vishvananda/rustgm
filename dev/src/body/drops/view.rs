//! Deterministic observations of pending cleanup facts, never proof input.
use super::{Pending, Trigger};
use crate::body::{
    ir::Program,
    storage::{End, Location},
};
use std::io::{self, Write};

pub(crate) fn write(program: &Program, pending: &Pending, out: &mut impl Write) -> io::Result<()> {
    for method in &pending.methods {
        writeln!(
            out,
            "pending-destructor item={} function={} target={} signature={} captures={} source={}:{}",
            method.item.0,
            method.function.0,
            method.target.0,
            method.signature.0,
            method.captures,
            method.source.file.0,
            method.source.node.get()
        )?;
    }
    for (index, facts) in pending.bodies.iter().enumerate() {
        if let Some(owned) = program
            .bodies
            .get(index)
            .and_then(|body| body.owned.as_ref())
        {
            write_owned(index, owned, out)?;
        }
        write!(out, "drop-requirements body={index} locals=")?;
        selected(&facts.locals, "l", out)?;
        write!(out, " values=")?;
        selected(&facts.values, "v", out)?;
        write!(out, " projections=")?;
        selected(&facts.projections, "p", out)?;
        writeln!(out)?;
        for site in &facts.sites {
            write!(out, "drop-candidate body={index} ")?;
            match site.location {
                Location::At { block, before } => write!(out, "at=b{}:{before} ", block.0)?,
                Location::Edge { block, target } => {
                    write!(out, "edge=b{}:b{} ", block.0, target.0)?;
                }
            }
            match site.trigger {
                Trigger::Locals { first, last } => write!(out, "locals=l{}:l{}", first.0, last.0)?,
                Trigger::Store(place) => {
                    write!(out, "store=")?;
                    crate::body::body_view::place(place, out)?;
                }
                Trigger::Exit(exit) => {
                    let exit = program
                        .bodies
                        .get(index)
                        .and_then(|b| b.storage.as_ref())
                        .and_then(|s| s.exits.get(exit as usize))
                        .ok_or_else(|| io::Error::other("invalid cleanup exit"))?;
                    write_exit(exit.end, out)?;
                }
            }
            writeln!(out, " pending")?;
        }
    }
    Ok(())
}
fn write_owned(index: usize, facts: &super::values::Facts, out: &mut impl Write) -> io::Result<()> {
    for value in &facts.created {
        write!(
            out,
            "owned-value body={index} value=v{} scope=",
            value.value.0
        )?;
        match value.lifetime {
            crate::body::scopes::Lifetime::Scope(id) => write!(out, "{}", id.0)?,
            crate::body::scopes::Lifetime::Static => write!(out, "static")?,
        }
        writeln!(
            out,
            " birth=b{}:{} exits-before={} source={}",
            value.block.0,
            value.before,
            value.exits_before,
            value.source.get()
        )?;
    }
    for release in &facts.returned {
        write!(out, "return-owned body={index} value=v{} ", release.value.0)?;
        match release.location {
            Location::At { block, before } => write!(out, "at=b{}:{before}", block.0)?,
            Location::Edge { block, target } => write!(out, "edge=b{}:b{}", block.0, target.0)?,
        }
        writeln!(out, " exits-before={}", release.exits_before)?;
    }
    Ok(())
}
fn write_exit(end: End, out: &mut impl Write) -> io::Result<()> {
    match end {
        End::Scopes(range) => {
            write!(out, "scopes={}:", range.inner.0)?;
            if let Some(outer) = range.outer {
                write!(out, "{}", outer.0)?;
            } else {
                write!(out, "-")?;
            }
        }
        End::Temporaries { start, end } => write!(out, "temporaries={start}:{end}")?,
    }
    Ok(())
}
fn selected(flags: &[bool], prefix: &str, out: &mut impl Write) -> io::Result<()> {
    write!(out, "[")?;
    let mut first = true;
    for (index, selected) in flags.iter().enumerate() {
        if *selected {
            if !first {
                write!(out, ",")?;
            }
            first = false;
            write!(out, "{prefix}{index}")?;
        }
    }
    write!(out, "]")
}
