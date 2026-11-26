//! Reuse checked lexical policy to publish sparse declaration deprecation.
use super::{Declarations, Report, Stage, s};
use crate::{
    resolve::{
        lint_policy::{SourceDeprecation, producer::Tail},
        model::{DefId, DefKind},
        unit::NodeKey,
    },
    support::{crates::owner_catalog::DeclarationRef, hash::HashMap},
    syntax::{ast::Kind, item::ItemKind},
};

struct Origin<'a> {
    source: SourceDeprecation<'a>,
    declaration: Option<DeclarationRef>,
}

impl Stage {
    pub(super) fn deprecations(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<Option<Box<s::DeprecationColumns>>, &'static str> {
        let Some(producer) = report.lint_producer() else {
            return Ok(None);
        };
        let mut origins = HashMap::default();
        for source in producer.source_deprecations(&report.package) {
            let source = source.map_err(|error| error.message)?;
            let key = source.declaration;
            let ast = &report.package.unit(key.file).configured.ast;
            if let Kind::Item(index) = ast.node(key.node).kind
                && matches!(
                    ast.items[index as usize].kind,
                    ItemKind::Use(_)
                        | ItemKind::MacroRules { .. }
                        | ItemKind::DeclarativeMacro { .. }
                )
            {
                // A use binding keeps the target's notice. Macro definitions own a separate table.
                continue;
            }
            self.reserve(4 * size_of::<(Tail, Origin<'_>)>())?;
            if origins
                .insert(
                    source.tail,
                    Origin {
                        source,
                        declaration: None,
                    },
                )
                .is_some()
            {
                return Err("source deprecation origin was recorded twice");
            }
        }
        if origins.is_empty() {
            return Ok(None);
        }
        let affected = self.deprecation_placements(report, facts, &mut origins)?;
        self.reserve(size_of::<s::DeprecationColumns>())?;
        let mut rows = Box::<s::DeprecationColumns>::default();
        for origin in origins.values() {
            rows.notices.push(self.deprecation_notice(origin)?);
        }
        for (declaration, tail) in affected {
            let owner = origins
                .get(&tail)
                .and_then(|origin| origin.declaration)
                .ok_or("source deprecation use lost its defining owner")?;
            self.reserve(4 * size_of::<s::DeprecatedDeclaration>())?;
            rows.declarations
                .push(s::DeprecatedDeclaration { declaration, owner });
        }
        for len in [rows.notices.len(), rows.declarations.len()] {
            self.charge(len.saturating_mul(len.checked_ilog2().unwrap_or(0) as usize + 1))?;
        }
        rows.notices.sort_unstable_by_key(|notice| notice.owner.0);
        rows.declarations
            .sort_unstable_by_key(|row| row.declaration.0);
        Ok(Some(rows))
    }

    fn deprecation_placements(
        &mut self,
        report: &Report,
        facts: &Declarations,
        origins: &mut HashMap<Tail, Origin<'_>>,
    ) -> Result<Vec<(DeclarationRef, Tail)>, &'static str> {
        let mut affected = Vec::new();
        for index in 0..self.definitions.len() {
            self.charge(1)?;
            let Some(declaration) = self.definitions[index] else {
                continue;
            };
            let actual = &report.resolution.definitions[index];
            if actual.kind.is_local() {
                continue;
            }
            let Some(key) = deprecation_key(report, facts, index)? else {
                continue;
            };
            let Some(tail) = facts.safety_policies.deprecated_origin(key) else {
                continue;
            };
            let origin = origins
                .get_mut(&tail)
                .ok_or("source deprecation tail is foreign")?;
            if (key == origin.source.key || key == origin.source.declaration)
                && actual.kind != DefKind::Constructor
                && origin.declaration.replace(declaration).is_some()
            {
                return Err("source deprecation origin has multiple declarations");
            }
            self.reserve(4 * size_of::<(DeclarationRef, Tail)>())?;
            affected.push((declaration, tail));
        }
        Ok(affected)
    }

    fn deprecation_notice(
        &mut self,
        origin: &Origin<'_>,
    ) -> Result<s::DeprecationNotice, &'static str> {
        let owner = origin.declaration.ok_or(
            "source deprecation origin requires its wider declaration or binding transport",
        )?;
        let source = &origin.source;
        let bytes = [source.since, source.note, source.suggestion]
            .into_iter()
            .flatten()
            .map(str::len)
            .sum::<usize>();
        self.charge(bytes.saturating_add(1))?;
        self.reserve(bytes.saturating_add(4 * size_of::<s::DeprecationNotice>()))?;
        Ok(s::DeprecationNotice {
            owner,
            site: self.site(owner, source.key)?,
            since: source.transport_since()?,
            note: source.note.map(Box::from),
            suggestion: source.suggestion.map(Box::from),
        })
    }
}

fn deprecation_key(
    report: &Report,
    facts: &Declarations,
    index: usize,
) -> Result<Option<NodeKey>, &'static str> {
    let actual = &report.resolution.definitions[index];
    if actual.kind != DefKind::Module {
        return Ok(actual.node);
    }
    let id = DefId(u32::try_from(index).map_err(|_| "deprecation module identity overflow")?);
    let module =
        crate::resolve::impl_restrictions::module(&report.package, &report.resolution, id)?;
    let module = &report.package.modules[module.0 as usize];
    let root = NodeKey {
        file: module.file,
        node: module.root,
    };
    Ok(if facts.safety_policies.deprecated_origin(root).is_some() {
        Some(root)
    } else {
        actual.node
    })
}
