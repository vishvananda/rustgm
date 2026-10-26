//! Helper names are declaration data, scoped to a selected derive by its consumer.
use super::{HelperAttribute, Owner, Package, State, lists::Builder};
use crate::{
    lex::token::Punctuation,
    resolve::unit::editions,
    syntax::{attribute, token_tree::Forest},
};
impl Builder {
    pub(super) fn parse_helpers(
        &mut self,
        package: &Package,
        owner: &Owner<'_>,
        forest: &Forest,
        effect: usize,
        range: std::ops::Range<u32>,
        work: &mut State,
    ) -> Result<(), &'static str> {
        work.charge((range.end - range.start) as usize, 0)?;
        let mut index = range.start;
        while index < range.end {
            let token = forest
                .token(index)
                .ok_or("builtin helper requires an identifier")?;
            let edition = editions::token(owner.node.file, token, |file| {
                package.units.get(file.0 as usize)
            })
            .map_err(|error| error.message)?;
            let name = forest
                .name(owner.unit.source.text(), index)
                .ok_or("builtin helper requires an identifier")?;
            if !attribute::word(forest, owner.unit.source.text(), index, edition)
                .map_err(|error| error.message)?
                || matches!(name, "_" | "self" | "Self" | "crate" | "super")
            {
                return Err("builtin helper requires an identifier that can be raw");
            }
            let span = forest
                .tree(index)
                .ok_or("builtin helper tree disappeared")?
                .span;
            if self
                .helpers
                .last()
                .is_some_and(|helper| helper.span().end > span.start)
            {
                return Err("builtin helper declaration order changed");
            }
            let growth = crate::support::retained::Growth::vector::<HelperAttribute>(
                self.helpers.len(),
                self.helpers.capacity(),
                1,
            );
            work.charge(growth.work, growth.bytes)?;
            let name = self.symbol(name, work)?;
            self.helpers
                .try_reserve(1)
                .map_err(|_| "builtin helper allocation failed")?;
            self.helpers.push(HelperAttribute::new(name, span));
            self.check_storage(work)?;
            index += 1;
            if index == range.end {
                break;
            }
            if !forest.punctuation(index, Punctuation::Comma) {
                return Err("invalid builtin helper separator");
            }
            index += 1;
        }
        self.helper_input = Some((effect, range));
        Ok(())
    }
}
impl crate::resolve::expansion::Report {
    /// Visit declaration data from the actual published local macro. This is
    /// diagnostic metadata, never permission to execute a derive or use a helper.
    pub(crate) fn visit_builtin_helpers(
        &self,
        definition: crate::resolve::model::DefId,
        mut visit: impl FnMut(
            crate::resolve::unit::NodeKey,
            &str,
            crate::syntax::token_tree::Span,
            crate::expand::marks::ExpansionId,
        ) -> Result<(), crate::resolve::package::Error>,
    ) -> Result<(), crate::resolve::package::Error> {
        use crate::resolve::{
            model::{DefKind, Namespace},
            package::Error,
        };
        let actual = self
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| Error::unlocated("helper diagnostic declaration disappeared"))?;
        let key = actual
            .node
            .ok_or_else(|| Error::unlocated("helper diagnostic requires an actual source macro"))?;
        let error = |message| Error::at(key.file, 0, message);
        let scope = self
            .resolution
            .scopes
            .get(actual.scope.0 as usize)
            .ok_or_else(|| error("helper diagnostic macro scope disappeared"))?;
        if actual.kind != DefKind::Macro
            || self
                .package
                .modules
                .get(scope.module.0 as usize)
                .is_none_or(|module| module.file != key.file)
            || self
                .resolution
                .item_definitions
                .get(&(scope.module, key.node, Namespace::Macro))
                != Some(&definition)
            || self.resolution.node_scopes.get(&(scope.module, key.node)) != Some(&actual.scope)
        {
            return Err(error("helper diagnostic macro identity changed"));
        }
        let Some(header) = super::checked_definition(&self.package, key)? else {
            return Ok(());
        };
        let summary = header.summary();
        if summary.node != key {
            return Err(error("helper diagnostic header belongs to another macro"));
        }
        if summary.builtin.is_none() {
            return Ok(());
        }
        let data = header
            .into_data()
            .ok_or_else(|| error("builtin helper declaration data is missing"))?;
        let context = self
            .package
            .macro_context()
            .map_err(error)?
            .ok_or_else(|| error("builtin helper provider is missing"))?;
        let introduction = self
            .resolution
            .introductions
            .definitions
            .get(&definition)
            .copied()
            .unwrap_or(crate::expand::marks::ExpansionId(0));
        for helper in &data.helpers {
            context
                .contracts()
                .charge(
                    crate::support::helper_collisions::comparison_work(helper.name()),
                    0,
                )
                .map_err(error)?;
            visit(
                summary.source_owner(),
                helper.name(),
                helper.span(),
                introduction,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/derive_headers.rs"]
mod tests;
