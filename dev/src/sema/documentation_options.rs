//! Documentation predicates validate syntax/authority without configuring code.
use crate::{
    lex::token::Punctuation,
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    syntax::{
        attribute::{Attribute, Input},
        configuration::CfgGate,
        token_tree::{Delimiter, Forest, TreeKind, TreeRange},
    },
};

#[derive(Clone, Copy)]
enum Gate {
    Configuration,
    Notable,
}
impl Gate {
    const fn name(self) -> &'static str {
        match self {
            Self::Configuration => "doc_cfg",
            Self::Notable => "doc_notable_trait",
        }
    }
}
/// Cold pass-local permission caches never replace each occurrence's authority.
#[derive(Default)]
pub(super) struct Draft {
    gates: [Option<bool>; 2],
    queries: [Option<bool>; CfgGate::COUNT],
}
impl Draft {
    fn require(
        &mut self,
        report: &Report,
        key: NodeKey,
        attribute: Attribute,
        gate: Gate,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let error = |message| Error::at(key.file, attribute.span.start, message);
        let enabled = if let Some(enabled) = self.gates[gate as usize] {
            enabled
        } else {
            let enabled = report
                .package
                .requested_documentation(matches!(gate, Gate::Notable))
                .map_err(error)?;
            self.gates[gate as usize] = Some(enabled);
            enabled
        };
        if !enabled
            && !allowances.allows_at(
                key.file,
                attribute.span,
                &FeatureName::new(gate.name()).map_err(error)?,
            )?
        {
            return Err(error(
                "documentation option requires its feature or actual macro allowance",
            ));
        }
        Ok(())
    }

    pub(super) fn consume(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        (owner, key): (NodeKey, NodeKey),
        (attribute, retained): (Attribute, usize),
        forest: &Forest,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let error = |message| Error::at(key.file, attribute.span.start, message);
        if attribute.unsafe_wrapper {
            return Err(error("doc cannot use an unsafe attribute wrapper"));
        }
        let Input::Delimited(group) = attribute.input else {
            return Ok(());
        };
        let range = forest
            .children(group)
            .ok_or_else(|| error("missing doc option list"))?;
        let source = report.package.unit(key.file).source.text();
        let mut cursor = range.start;
        let mut gated = false;
        while cursor < range.end {
            let start = cursor;
            while cursor < range.end && !forest.punctuation(cursor, Punctuation::Comma) {
                cursor = forest.after(cursor);
            }
            let end = cursor;
            cursor += u32::from(cursor < range.end);
            let name = forest.name(source, start);
            if name == Some("masked") {
                crate::sema::header_attributes::reserve_source_contracts(
                    facts,
                    1 + (end - start) as usize,
                    0,
                )?;
                Self::masked(
                    report,
                    facts,
                    (owner, key),
                    (attribute, retained),
                    end == start + 1,
                    allowances,
                )?;
                continue;
            }
            let gate = match name {
                Some("cfg" | "auto_cfg") => Gate::Configuration,
                Some("notable_trait") => Gate::Notable,
                _ => continue,
            };
            crate::sema::header_attributes::reserve_source_contracts(
                facts,
                1 + (end - start) as usize,
                0,
            )?;
            self.require(report, key, attribute, gate, allowances)?;
            gated = true;
            if name == Some("cfg") {
                let input = predicate(forest, start, end).map_err(error)?;
                let inputs = origins::Inputs {
                    report,
                    file: key.file,
                };
                let uses =
                    crate::syntax::configuration::documentation(forest, source, input, &inputs)
                        .map_err(|failure| Error::at(key.file, failure.offset, failure.message))?;
                for used in uses {
                    self.require_query(report, key, used, allowances)?;
                }
            }
        }
        if gated {
            facts.require_lint_context(report)?;
            if !report.package.lint_cap().suppresses_lints() {
                return Err(error(
                    "uncapped documentation option diagnostics are not implemented",
                ));
            }
        }
        Ok(())
    }

    fn masked(
        report: &Report,
        facts: &mut Declarations,
        (owner, declaration): (NodeKey, NodeKey),
        (attribute, retained): (Attribute, usize),
        word: bool,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let error = |message| Error::at(owner.file, attribute.span.start, message);
        if !report.package.requested_doc_masked(owner.file)?
            && !allowances.allows_attribute(
                owner,
                retained,
                attribute,
                &FeatureName::new("doc_masked").map_err(error)?,
            )?
        {
            return Err(error(
                "doc(masked) requires this crate's doc_masked feature or actual macro allowance",
            ));
        }
        if !word {
            return Err(error("doc(masked) takes no arguments"));
        }
        facts.require_lint_context(report)?;
        let unit = report.package.unit(declaration.file);
        let ast = &unit.configured.ast;
        let item = match ast.node(declaration.node).kind {
            crate::syntax::ast::Kind::Item(index) => ast.items.get(index as usize),
            _ => None,
        };
        let message = if let Some(item) = item
            .filter(|item| matches!(item.kind, crate::syntax::item::ItemKind::ExternCrate { .. }))
        {
            let source_name = item
                .name
                .ok_or_else(|| error("extern crate has no source name"))?;
            if unit
                .spelling(source_name)
                .map_err(|failure| error(failure.message))?
                != "self"
            {
                // Resolution already established the configured dependency.
                // Masking does not change its binding or its exports.
                return Ok(());
            }
            "this attribute cannot be applied to an `extern crate self` item"
        } else {
            "this attribute can only be applied to an `extern crate` item"
        };
        facts.must_use_diagnostic_in(
            report,
            owner,
            attribute.span.start,
            0,
            crate::support::diagnostics::WarningCode::InvalidDocAttributes,
            (message.into(), crate::resolve::model::ExpansionId(0)),
        )
    }

    fn require_query(
        &mut self,
        report: &Report,
        key: NodeKey,
        used: crate::syntax::configuration::CfgUse,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let error = |message| Error::at(key.file, used.span.start, message);
        let enabled = if let Some(enabled) = self.queries[used.gate as usize] {
            enabled
        } else {
            let enabled = report
                .package
                .requested_cfg_gate(used.gate)
                .map_err(error)?;
            self.queries[used.gate as usize] = Some(enabled);
            enabled
        };
        if !enabled
            && !allowances.allows_at(key.file, used.span, &FeatureName::for_cfg(used.gate))?
        {
            return Err(error(
                "documentation predicate requires its cfg feature or actual macro allowance",
            ));
        }
        Ok(())
    }
}

fn predicate(forest: &Forest, start: u32, end: u32) -> Result<TreeRange, &'static str> {
    if start + 1 >= end
        || forest.after(start + 1) != end
        || !matches!(
            forest.tree(start + 1).map(|tree| tree.kind),
            Some(TreeKind::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            })
        )
    {
        return Err("doc(cfg) requires one parenthesized predicate");
    }
    forest
        .children(start + 1)
        .ok_or("missing documentation predicate")
}

mod origins;
