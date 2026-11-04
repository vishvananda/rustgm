//! Unsized ownership is restricted to actual outermost function parameters.
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::{hash::HashSet, retained},
    syntax::ast::Kind,
};
#[derive(Default)]
pub(super) struct Draft {
    enabled: Option<bool>,
    direct: Option<Box<HashSet<DefId>>>,
}
impl Draft {
    pub(super) fn enabled(&mut self, report: &Report) -> Result<bool, Error> {
        if let Some(enabled) = self.enabled {
            return Ok(enabled);
        }
        let enabled = report
            .package
            .requested_unsized_parameters()
            .map_err(Error::unlocated)?;
        self.enabled = Some(enabled);
        Ok(enabled)
    }
    pub(super) fn binding(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        id: DefId,
    ) -> Result<bool, Error> {
        if report
            .resolution
            .definitions
            .get(id.0 as usize)
            .is_none_or(|d| d.kind != DefKind::Parameter)
            || !self.enabled(report)?
        {
            return Ok(false);
        }
        if self.direct.is_none() {
            let mut count = 0_usize;
            for function in &facts.functions {
                count = count.saturating_add(
                    report
                        .package
                        .unit(function.node.file)
                        .configured
                        .ast
                        .list(function.parameters)
                        .len(),
                );
            }
            super::header_attributes::reserve_source_contracts(
                facts,
                count.saturating_add(facts.functions.len()),
                retained::new_map::<DefId, ()>(count),
            )?;
            let mut direct = HashSet::default();
            direct
                .try_reserve(count)
                .map_err(|_| Error::unlocated("unsized parameter index allocation failed"))?;
            for function in &facts.functions {
                let unit = report.package.unit(function.node.file);
                for &node in unit.configured.ast.list(function.parameters) {
                    if !unit.configured.active[node.get() as usize] {
                        continue;
                    }
                    let node = plain(
                        report,
                        NodeKey {
                            node,
                            ..function.node
                        },
                    );
                    if let Some(receiver) = facts.receivers.get(&node) {
                        direct.insert(receiver.definition);
                        continue;
                    }
                    let Kind::Parameter { pattern, .. } = unit.configured.ast.node(node.node).kind
                    else {
                        continue;
                    };
                    let pattern = plain(
                        report,
                        NodeKey {
                            node: pattern,
                            ..node
                        },
                    );
                    if matches!(
                        unit.configured.ast.node(pattern.node).kind,
                        Kind::Binding { by_ref: false, .. }
                    ) && let Some(id) = facts.node(pattern).and_then(|f| f.binding)
                    {
                        direct.insert(id);
                    }
                }
            }
            self.direct = Some(Box::new(direct));
        }
        Ok(self.direct.as_ref().is_some_and(|d| d.contains(&id)))
    }
}
fn plain(report: &Report, mut key: NodeKey) -> NodeKey {
    let ast = &report.package.unit(key.file).configured.ast;
    while let Kind::Attributed { value, .. } | Kind::Group(value) = ast.node(key.node).kind {
        key.node = value;
    }
    key
}

#[cfg(test)]
#[path = "../../tests/invariants/unsized_parameters.rs"]
mod tests;
