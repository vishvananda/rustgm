//! Transient source optimization and advisory contracts never grant an effect.
use super::{
    Attribute, Declarations, DefId, DefKind, Definition, Error, Forest, MacroAllowanceSession,
    NodeKey, Report, reserve_source_contracts,
};
use crate::syntax::{ast::Kind, item::ItemKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Restriction {
    MirInline,
    Writable,
    ConstItemCalls,
    Confusables,
    Suspend,
}
impl Restriction {
    fn named(name: &str) -> Option<Self> {
        match name {
            "rustc_no_mir_inline" => Some(Self::MirInline),
            "rustc_no_writable" => Some(Self::Writable),
            "rustc_should_not_be_called_on_const_items" => Some(Self::ConstItemCalls),
            "rustc_confusables" => Some(Self::Confusables),
            "must_not_suspend" => Some(Self::Suspend),
            _ => None,
        }
    }
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::MirInline => "rustc_no_mir_inline",
            Self::Writable => "rustc_no_writable",
            Self::ConstItemCalls => "rustc_should_not_be_called_on_const_items",
            Self::Confusables => "rustc_confusables",
            Self::Suspend => "must_not_suspend",
        }
    }
    pub fn duplicate_error(self) -> &'static str {
        match self {
            Self::MirInline => "duplicate rustc_no_mir_inline attribute",
            Self::Writable => "duplicate rustc_no_writable attribute",
            Self::ConstItemCalls => "duplicate rustc_should_not_be_called_on_const_items attribute",
            Self::Confusables => "confusables lists combine rather than reject repeated attributes",
            Self::Suspend => "duplicate must_not_suspend attribute",
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
            Self::ConstItemCalls => {
                "rustc_should_not_be_called_on_const_items requires its actual inherent or trait-impl method"
            }
            Self::Confusables => "rustc_confusables requires its actual inherent method",
            Self::Suspend => "must_not_suspend requires its actual struct, enum, union or trait",
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
                ItemKind::Struct { .. }
                | ItemKind::Enum { .. }
                | ItemKind::Union { .. }
                | ItemKind::Trait { .. }
                    if self == Self::Suspend =>
                {
                    Some(false)
                }
                ItemKind::Function { body, .. }
                    if self != Self::Suspend && (self == Self::MirInline || body.is_some()) =>
                {
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
    let error = |message: &str| Error::at(declaration.file, attribute.span.start, message);
    let unit = report.package.unit(declaration.file);
    let restriction = attribute
        .name(forest, unit.source.text())
        .and_then(Restriction::named)
        .ok_or_else(|| error("unknown source optimization restriction"))?;
    validate_site(
        report,
        facts,
        (owner, declaration),
        (attribute, effect),
        restriction,
    )?;
    super::advisory_inputs::validate(report, facts, declaration, attribute, forest, restriction)?;
    let (feature, requested) = if restriction == Restriction::Suspend {
        (
            "must_not_suspend",
            report.package.requested_must_not_suspend().map_err(error)?,
        )
    } else {
        (
            "rustc_attrs",
            report.package.requested_internal_attributes(),
        )
    };
    // The attribute envelope retains the actual authored `#`, including an
    // allowing macro's meta substitution, without granting caller-name privilege.
    if !requested
        && !allowances.allows_at(
            declaration.file,
            attribute.span,
            &crate::resolve::bindings::FeatureName::new(feature).map_err(error)?,
        )?
    {
        return Err(error(&format!(
            "{} requires its {feature} request",
            restriction.name()
        )));
    }
    let closure = restriction.target(unit, declaration.node).ok_or_else(|| {
        error(&format!(
            "{} requires an allowed function, method or closure target",
            restriction.name()
        ))
    })?;
    if closure
        && !report
            .package
            .requested_expression_attributes()
            .map_err(error)?
        && !allowances.allows_at(
            declaration.file,
            attribute.span,
            &crate::resolve::bindings::FeatureName::new("stmt_expr_attributes").map_err(error)?,
        )?
    {
        return Err(error(
            "source closure attributes require stmt_expr_attributes",
        ));
    }
    Ok((restriction, closure))
}

/// The marker is accepted on impl methods, including associated functions, but
/// not free functions or declarations/default bodies belonging to a trait.
pub(super) fn validate_impl_method(
    report: &Report,
    facts: &Declarations,
    id: DefId,
    definition: &Definition,
    inherent: bool,
) -> Result<(), Error> {
    let owner = facts
        .associated
        .owner(id)
        .and_then(|id| facts.associated.get(id))
        .ok_or_else(|| Error::unlocated("const-item advisory requires an actual impl method"))?;
    let crate::sema::associated::OwnerKind::Impl {
        target, trait_type, ..
    } = owner.kind
    else {
        return Err(Error::unlocated(
            "const-item advisory requires an actual impl method",
        ));
    };
    if inherent && trait_type.is_some() {
        return Err(Error::unlocated(
            "rustc_confusables requires an inherent method, not a trait implementation",
        ));
    }
    let source = owner
        .source
        .ok_or_else(|| Error::unlocated("const-item advisory impl source missing"))?;
    let unit = report.package.unit(source.file);
    let self_definition = report
        .resolution
        .definitions
        .get(owner.self_definition.0 as usize);
    if self_definition.is_none_or(|row| {
        row.kind != DefKind::SelfType || row.node != Some(source) || row.scope != definition.scope
    }) || unit.configured.active.get(source.node.get() as usize) != Some(&true)
        || target.file != source.file
        || trait_type.is_some_and(|key| key.file != source.file)
        || !matches!(unit.configured.ast.node(source.node).kind, Kind::Item(index)
            if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::Impl { ty, trait_path, .. }
                if ty == target.node && trait_path == trait_type.map(|key| key.node)))
    {
        return Err(Error::unlocated(
            "const-item advisory changed its actual impl owner, scope or source",
        ));
    }
    Ok(())
}

impl Restriction {
    pub(super) fn validate_definition(
        self,
        report: &Report,
        facts: &Declarations,
        actual: DefId,
        definition: &Definition,
        key: NodeKey,
    ) -> Result<(), Error> {
        let unit = report.package.unit(key.file);
        let allowed = if self == Self::Suspend {
            definition.namespace == super::Namespace::Type
                && matches!(unit.configured.ast.node(key.node).kind, Kind::Item(index)
                if matches!((definition.kind, unit.configured.ast.items[index as usize].kind),
                    (DefKind::Struct, ItemKind::Struct { .. } | ItemKind::Union { .. })
                    | (DefKind::Enum, ItemKind::Enum { .. }) | (DefKind::Trait, ItemKind::Trait { .. })))
        } else {
            definition.kind == DefKind::Function
                && definition.namespace == super::Namespace::Value
                && self.target(unit, key.node) == Some(false)
        };
        if !allowed
            || report.resolution.foreign_owners.contains_key(&actual)
            || unit.configured.active.get(key.node.get() as usize) != Some(&true)
        {
            return Err(Error::unlocated(self.binding_error()));
        }
        if matches!(self, Self::ConstItemCalls | Self::Confusables) {
            validate_impl_method(report, facts, actual, definition, self == Self::Confusables)?;
        }
        Ok(())
    }
}

fn validate_site(
    report: &Report,
    facts: &mut Declarations,
    (owner, declaration): (NodeKey, NodeKey),
    (attribute, effect): (Attribute, usize),
    restriction: Restriction,
) -> Result<(), Error> {
    let unit = report.package.unit(declaration.file);
    let error = |message: &str| Error::at(declaration.file, attribute.span.start, message);
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
    Ok(())
}

/// An auxiliary value constructor or trait Self can share an item source node;
/// the resolver's existing canonical item index remains the completion owner.
pub(super) fn nominal_auxiliary(
    report: &Report,
    facts: &Declarations,
    actual: DefId,
    definition: &Definition,
    key: NodeKey,
) -> Result<bool, Error> {
    let error = || Error::unlocated("suspension advisory lost its canonical type declaration");
    let module = report
        .resolution
        .scopes
        .get(definition.scope.0 as usize)
        .ok_or_else(error)?
        .module;
    let canonical = *report
        .resolution
        .item_definitions
        .get(&(module, key.node, super::Namespace::Type))
        .ok_or_else(error)?;
    if actual == canonical {
        return Ok(false);
    }
    let declaration = report
        .resolution
        .definitions
        .get(canonical.0 as usize)
        .filter(|row| row.node == Some(key))
        .ok_or_else(error)?;
    Restriction::Suspend.validate_definition(report, facts, canonical, declaration, key)?;
    let valid = match definition.kind {
        DefKind::Constructor => definition.namespace == super::Namespace::Value
            && definition.scope == declaration.scope
            && report.resolution.item_definitions.get(&(module, key.node, super::Namespace::Value)) == Some(&actual),
        DefKind::SelfType => definition.namespace == super::Namespace::Type
            && declaration.members == Some(definition.scope)
            && facts.associated.owner(actual).and_then(|id| facts.associated.get(id)).is_some_and(|owner| {
                owner.self_definition == actual && owner.source == Some(key)
                    && matches!(owner.kind, crate::sema::associated::OwnerKind::Trait { definition, .. } if definition == canonical)
            }),
        _ => false,
    };
    if !valid {
        return Err(error());
    }
    Ok(true)
}
