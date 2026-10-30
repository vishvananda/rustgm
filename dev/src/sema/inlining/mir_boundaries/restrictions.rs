//! Source restrictions suppress distinct optimizations; neither grants an effect.
use super::*;
use crate::syntax::{ast::Kind, attribute::Input, item::ItemKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Restriction {
    MirInline,
    Writable,
}
impl Restriction {
    pub const ALL: [Self; 2] = [Self::MirInline, Self::Writable];
    fn named(name: &str) -> Option<Self> {
        match name {
            "rustc_no_mir_inline" => Some(Self::MirInline),
            "rustc_no_writable" => Some(Self::Writable),
            _ => None,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::MirInline => "rustc_no_mir_inline",
            Self::Writable => "rustc_no_writable",
        }
    }
    pub fn duplicate_error(self) -> &'static str {
        match self {
            Self::MirInline => "duplicate rustc_no_mir_inline attribute",
            Self::Writable => "duplicate rustc_no_writable attribute",
        }
    }
    pub fn binding_error(self) -> &'static str {
        match self {
            Self::MirInline => {
                "rustc_no_mir_inline requires its actual non-foreign function or method"
            }
            Self::Writable => {
                "rustc_no_writable requires its actual non-foreign function or method body"
            }
        }
    }
    fn target(
        self,
        unit: &crate::resolve::unit::Unit,
        node: crate::syntax::ast::NodeId,
    ) -> Option<bool> {
        let ast = &unit.configured.ast;
        match ast.node(node).kind {
            Kind::Closure { .. } if self == Self::Writable => Some(true),
            Kind::Item(index) => match ast.items[index as usize].kind {
                ItemKind::Function { body, .. } if self == Self::MirInline || body.is_some() => {
                    Some(false)
                }
                _ => None,
            },
            _ => None,
        }
    }
}

pub(super) fn authenticate(
    report: &Report,
    facts: &mut Declarations,
    (owner, declaration): (NodeKey, NodeKey),
    (attribute, effect): (Attribute, usize),
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(Restriction, bool), Error> {
    facts.require_lint_context(report)?;
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    let unit = report.package.unit(declaration.file);
    let restriction = attribute
        .name(forest, unit.source.text())
        .and_then(Restriction::named)
        .ok_or_else(|| error("unknown source optimization restriction"))?;
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error(&format!(
            "{} requires a plain word attribute",
            restriction.name()
        )));
    }
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| error("optimization boundary lost its actual name"))?
        .span;
    if !report.package.requested_internal_attributes()
        && !allowances.allows_at(
            declaration.file,
            name,
            &crate::resolve::bindings::FeatureName::new("rustc_attrs").map_err(error)?,
        )?
    {
        return Err(error(&format!(
            "{} requires its rustc_attrs request",
            restriction.name()
        )));
    }
    if owner.file != declaration.file
        || unit
            .owner_indices
            .get(owner.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
            .is_none_or(|row| {
                !row.enabled || row.node != owner.node || !row.attributes.contains(&effect)
            })
    {
        return Err(error(&format!(
            "{} requires its active configured owner",
            restriction.name()
        )));
    }
    let closure = restriction
        .target(unit, declaration.node)
        .ok_or_else(|| {
            error(&format!(
                "{} requires an allowed function, method or closure target",
                restriction.name()
            ))
        })?;
    let actual = unit
        .retained_attributes(effect..effect + 1)
        .next()
        .ok_or_else(|| error("optimization boundary lost its configured effect"))?;
    if actual.0 != attribute
        || actual.0.name(actual.1, unit.source.text()) != Some(restriction.name())
    {
        return Err(error("optimization boundary changed its actual attribute"));
    }
    let mut node = owner.node;
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
        reserve_source_contracts(facts, 1, 0)?;
        node = value;
    }
    if node != declaration.node {
        return Err(error(
            "optimization boundary changed its actual declaration",
        ));
    }
    Ok((restriction, closure))
}
