//! Deterministic diagnostic view of typed name facts; never consumed by semantics.
use super::{
    expansion::{Body, Report},
    model::{Access, Resolution, Target},
    names::NameId,
    package::Package,
};
use crate::{expand::crate_syntax, literal, syntax::expression};
use std::io::{self, Write};
/// # Errors
/// Propagates output failures.
pub fn write(package: &Package, facts: &Resolution, out: &mut impl Write) -> io::Result<()> {
    for index in 0..package.names.len() {
        let id = NameId(u32::try_from(index).map_err(|_| io::Error::other("too many names"))?);
        writeln!(out, "name {index} {:?}", package.names.get(id))?;
    }
    for (index, module) in package.modules.iter().enumerate() {
        write!(
            out,
            "module {index} name={} file={} scope={} parent=",
            module.name.0, module.file.0, facts.module_scopes[index].0
        )?;
        if let Some(parent) = module.parent {
            write!(out, "{}", parent.0)?;
        } else {
            write!(out, "root")?;
        }
        writeln!(out)?;
    }
    for (index, scope) in facts.scopes.iter().enumerate() {
        write!(
            out,
            "scope {index} module={} barrier={} parent=",
            scope.module.0, scope.barrier
        )?;
        if let Some(parent) = scope.parent {
            write!(out, "{}", parent.0)?;
        } else {
            write!(out, "root")?;
        }
        writeln!(out)?;
    }
    for (index, definition) in facts.definitions.iter().enumerate() {
        write!(
            out,
            "def {index} {} {} name={} scope={} access=",
            definition.namespace.name(),
            definition.kind.name(),
            definition.name.0,
            definition.scope.0
        )?;
        access(out, definition.access)?;
        if let Some(node) = definition.node {
            write!(out, " source={}:{}", node.file.0, node.node.0)?;
        }
        if let Some(members) = definition.members {
            write!(out, " members={}", members.0)?;
        }
        writeln!(out)?;
    }
    for (index, scope) in facts.scopes.iter().enumerate() {
        for key in &scope.order {
            let Some(binding) = scope.bindings.get(key) else {
                continue;
            };
            write!(
                out,
                "binding scope={index} {} name={} def={} {:?} ambiguous={} access=",
                key.1.name(),
                key.0.0,
                binding.target.0,
                binding.origin,
                binding.ambiguous
            )?;
            access(out, binding.access)?;
            writeln!(out)?;
        }
    }
    for reference in &facts.references {
        write!(
            out,
            "ref {}:{} scope={} {} ",
            reference.node.file.0,
            reference.node.node.0,
            reference.scope.0,
            reference.namespace.name()
        )?;
        match reference.target {
            Target::Definition(id) => writeln!(out, "def={}", id.0)?,
            Target::Associated(id) => writeln!(out, "associated={}", id.0)?,
            Target::Expansion(scope) => writeln!(out, "pending-expansion={}", scope.0)?,
        }
    }
    for (scope, node) in &facts.expansions {
        writeln!(
            out,
            "expansion {}:{} scope={}",
            node.file.0, node.node.0, scope.0
        )?;
    }
    writeln!(
        out,
        "eof files={} modules={} definitions={} references={}",
        package.units.len(),
        package.modules.len(),
        facts.definitions.len(),
        facts.references.len()
    )
}
fn access(out: &mut impl Write, value: Access) -> io::Result<()> {
    match value {
        Access::Public => write!(out, "public"),
        Access::ExternalPrivate => write!(out, "external-private"),
        Access::Within(module) => write!(out, "module:{}", module.0),
    }
}

/// # Errors
/// Reports output failures without changing compiler facts.
pub fn count_expanded(report: &Report, out: &mut impl Write) -> io::Result<()> {
    writeln!(
        out,
        "files {} modules {} macros {} invocations {} captures {} definitions {} references {}",
        report.package.units.len(),
        report.package.modules.len(),
        report.definitions.len(),
        report.invocations.len(),
        report.captures.len(),
        report.resolution.definitions.len(),
        report.resolution.references.len()
    )
}

/// Draft PA8 diagnostic vocabulary. Earlier syntax and name views are reused
/// directly; no output text is a compiler input or an intermediate representation.
/// # Errors
/// Reports invalid literal storage or output failures.
pub fn expanded(report: &Report, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "expanded-crate 1")?;
    for (index, definition) in report.definitions.iter().enumerate() {
        write!(
            out,
            "macro {index} name={} module={} context={} introduction={} source=",
            definition.name.0, definition.module.0, definition.context.0, definition.introduction.0
        )?;
        if let Some(node) = definition.node {
            write!(out, "{}:{}", node.file.0, node.node.0)?;
        } else {
            write!(out, "builtin")?;
        }
        match &definition.body {
            Body::Declarative(compiled) => writeln!(
                out,
                " rules={} local-inner={}",
                compiled.definition.rules.len(),
                definition.local_inner_macros
            )?,
            Body::Builtin(builtin) => writeln!(out, " builtin={}", builtin.name())?,
        }
    }
    for (index, invocation) in report.invocations.iter().enumerate() {
        write!(
            out,
            "invocation {index} context={} parent={} macro={} source={}:{} original={}:{} generated={}:{} rule=",
            invocation.context.0,
            invocation.parent.0,
            invocation.definition,
            invocation.node.file.0,
            invocation.node.node.0,
            invocation.original.start,
            invocation.original.end,
            invocation.generated.start,
            invocation.generated.end
        )?;
        if let Some(rule) = invocation.rule {
            write!(out, "{rule}")?;
        } else {
            write!(out, "builtin")?;
        }
        writeln!(out, " parser-calls={}", invocation.parser_calls)?;
    }
    for (index, unit) in report.package.units.iter().enumerate() {
        writeln!(out, "file {index}")?;
        if let Some(context) = unit.include_context {
            writeln!(out, "include-template {context:?}")?;
        }
        crate_syntax::write_configuration(&unit.configured, out)?;
        expression::render::write_syntax(
            unit.source.text(),
            &unit.configured.ast,
            out,
            |index, span, out| {
                let value = unit
                    .literal(index)
                    .map_err(|e| io::Error::other(e.message))?;
                literal::render::value(out, &value, span.start, span.end)
            },
        )?;
        for origin in &unit.origins {
            write!(
                out,
                "origin {}:{} source={}:{}:{} context=",
                origin.generated.start,
                origin.generated.end,
                origin.original.file.0,
                origin.original.span.start,
                origin.original.span.end
            )?;
            if let Some(context) = origin.context {
                writeln!(out, "{}", context.0)?;
            } else {
                writeln!(out, "inherit")?;
            }
        }
        writeln!(out, "end-file {index}")?;
    }
    write(&report.package, &report.resolution, out)?;
    write!(out, "end-expanded-crate ")?;
    count_expanded(report, out)
}
