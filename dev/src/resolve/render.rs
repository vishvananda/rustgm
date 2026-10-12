//! Deterministic diagnostic view of typed name facts; never consumed by semantics.
use super::{
    model::{Access, Resolution, Target},
    names::NameId,
    package::Package,
};
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
        Access::Within(module) => write!(out, "module:{}", module.0),
    }
}
