//! Validate retained syntax once, then bind its contract to real canonical items.
use super::{Contract, Declarations, Error, HashMap, HashSet, NodeKey, Report, WarningCode};
use crate::{
    literal::{self, Value},
    resolve::{
        model::{DefId, Namespace},
        unit::FileId,
    },
    sema::non_exhaustive,
    support::decorations::Consumer,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};

pub(in crate::sema) fn collect(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    crate::sema::specialization::prepare(report, facts)?;
    let mut by_source = HashMap::default();
    let mut allowances = report.macro_allowances();
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(file).map_err(|_| Error::at(FileId(0), 0, "too many must_use files"))?,
        );
        for owner in &unit.configured.owners {
            let owner_key = NodeKey {
                file,
                node: owner.node,
            };
            if !owner.enabled {
                super::removed_owners::collect(report, facts, owner_key)?;
                continue;
            }
            let (key, supported) = declaration(unit, owner_key);
            crate::sema::header_attributes::observe_owner(report, facts, owner_key, key)?;
            crate::sema::specialization::observe_owner(report, facts, owner_key, key)?;
            let mut covered = false;
            for effect in owner.attributes.clone() {
                let Some((attribute, forest)) = unit.retained_attributes(effect..effect + 1).next()
                else {
                    continue;
                };
                if unit.derive_is_expanded(effect) {
                    report.expanded_derive(owner_key, effect)?;
                    continue;
                }
                if consume_annotation(
                    report,
                    facts,
                    (owner_key, key),
                    (attribute, effect),
                    forest,
                    &mut allowances,
                    &mut covered,
                )? {
                    continue;
                }
                facts.reserve_must_use_source(owner_key, attribute.span)?;
                let reason = reason(unit.source.text(), forest, attribute)
                    .map_err(|message| Error::at(file, attribute.span.start, message))?;
                if !supported {
                    facts.must_use_diagnostic(
                        report,
                        owner_key,
                        attribute.span.start,
                        0,
                        WarningCode::UnusedAttributes,
                        "must_use attribute requires a function, nominal type or trait".into(),
                    )?;
                    continue;
                }
                if by_source.contains_key(&key) {
                    facts.must_use_diagnostic(
                        report,
                        owner_key,
                        attribute.span.start,
                        0,
                        WarningCode::UnusedAttributes,
                        "duplicate must_use attribute".into(),
                    )?;
                    continue;
                }
                let index = retain(
                    facts,
                    Contract {
                        image: std::sync::Arc::clone(unit.source.original_image()),
                        owner: owner_key,
                        declaration: key,
                        span: attribute.span,
                        effect,
                        reason,
                    },
                )?;
                by_source.insert(key, index);
            }
        }
    }
    bind_definitions(report, facts, &by_source)?;
    crate::sema::specialization::finish(facts)?;
    complete_imports(report, facts)
}
fn retain(facts: &mut Declarations, contract: Contract) -> Result<usize, Error> {
    let contracts = facts.must_use_contracts.get_or_insert_default();
    if contracts.sources.len() >= 1_000_000 {
        return Err(Error::at(
            contract.owner.file,
            contract.span.start,
            "must_use contract budget exceeded",
        ));
    }
    let index = contracts.sources.len();
    contracts.work.attributes += 1;
    contracts.work.retained_contracts += 1;
    contracts.work.retained_reasons += contract.reason.as_ref().map_or(0, |reason| reason.len());
    contracts.sources.push(contract);
    Ok(index)
}
fn declaration(unit: &crate::resolve::unit::Unit, owner: NodeKey) -> (NodeKey, bool) {
    let mut node = owner.node;
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
        node = value;
    }
    let key = NodeKey { node, ..owner };
    let supported = matches!(unit.configured.ast.node(node).kind, Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind,
                ItemKind::Function { .. } | ItemKind::Struct { .. } | ItemKind::Enum { .. } | ItemKind::Union { .. } | ItemKind::Trait { .. }));
    (key, supported)
}
fn complete_imports(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    crate::sema::header_attributes::prepare_imported(report, facts)?;
    if let Some(imported) = &report.resolution.crates {
        let unknown: HashSet<_> = imported
            .definitions()
            .map(|(_, definition)| definition)
            .collect();
        if !unknown.is_empty() {
            facts.must_use_contracts.get_or_insert_default().unknown = unknown;
        }
    }
    Ok(())
}
fn consume_annotation(
    report: &Report,
    facts: &mut Declarations,
    (owner, declaration): (NodeKey, NodeKey),
    (attribute, effect): (Attribute, usize),
    forest: &Forest,
    allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
    covered: &mut bool,
) -> Result<bool, Error> {
    let consumer = attribute
        .name(forest, report.package.unit(declaration.file).source.text())
        .and_then(Consumer::named);
    match consumer {
        Some(Consumer::NoImplicitAutorefs) => {
            crate::sema::header_attributes::consume_no_implicit_autorefs(
                report,
                facts,
                declaration,
                attribute,
                forest,
                allowances,
            )?;
        }
        Some(Consumer::InheritedOverflow) => {
            crate::sema::header_attributes::consume_inherited_overflow(
                report,
                facts,
                declaration,
                attribute,
                forest,
                allowances,
            )?;
        }
        Some(Consumer::Coverage) => {
            if *covered {
                return Err(Error::at(
                    declaration.file,
                    attribute.span.start,
                    "duplicate coverage attribute",
                ));
            }
            *covered = true;
            crate::sema::coverage_annotations::consume(
                report,
                facts,
                declaration,
                attribute,
                forest,
                allowances,
            )?;
        }
        Some(Consumer::MarkerTrait) => {
            crate::sema::marker_traits::consume(
                report,
                facts,
                declaration,
                attribute,
                forest,
                allowances,
            )?;
        }
        _ => {
            return consume_other(
                report,
                facts,
                owner,
                declaration,
                attribute,
                effect,
                consumer,
            );
        }
    }
    Ok(true)
}
fn consume_other(
    report: &Report,
    facts: &mut Declarations,
    owner: NodeKey,
    declaration: NodeKey,
    attribute: Attribute,
    effect: usize,
    consumer: Option<Consumer>,
) -> Result<bool, Error> {
    match consumer {
        Some(Consumer::ConstPanicPolicy) => {
            crate::sema::header_attributes::consume_const_panic(
                report,
                facts,
                declaration,
                attribute,
            )?;
        }
        Some(
            consumer @ (Consumer::CallerTracking
            | Consumer::IntrinsicConstPolicy
            | Consumer::IntrinsicNoUnwind),
        ) => {
            crate::sema::header_attributes::consume_tracking(
                report,
                facts,
                declaration,
                attribute,
                consumer,
            )?;
        }
        Some(Consumer::DerivedImplementation) => {
            crate::sema::marker_traits::derived_impl(report, facts, declaration, attribute)?;
        }
        Some(Consumer::NonExhaustive) => {
            non_exhaustive::consume(report, facts, owner, declaration, attribute, effect)?;
        }
        Some(Consumer::UnsafeSpecialization) => {
            crate::sema::specialization::consume(
                report,
                facts,
                owner,
                declaration,
                attribute,
                effect,
            )?;
        }
        Some(Consumer::MustUse) => return Ok(false),
        _ => {}
    }
    Ok(true)
}
fn bind_definitions(
    report: &Report,
    facts: &mut Declarations,
    by_source: &HashMap<NodeKey, usize>,
) -> Result<(), Error> {
    if !by_source.is_empty()
        || report.package.source_core().is_some()
        || facts
            .must_use_contracts
            .as_deref()
            .is_some_and(|owner| owner.specialization.is_some())
    {
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            if definition.kind == crate::resolve::model::DefKind::Macro && definition.node.is_some()
            {
                report.visit_builtin_helpers(
                    DefId(
                        u32::try_from(index)
                            .map_err(|_| Error::unlocated("too many source macro definitions"))?,
                    ),
                    |owner, name, span, introduction| {
                        if crate::support::helper_collisions::helper_collision(name) {
                            facts.reserve_must_use_source(owner, span)?;
                            facts.must_use_diagnostic_in(
                                report,
                                owner,
                                span.start,
                                0,
                                WarningCode::AmbiguousDeriveHelpers,
                                (
                                    "there exists a built-in attribute with the same name".into(),
                                    introduction,
                                ),
                            )?;
                        }
                        Ok(())
                    },
                )?;
            }
            if let Some(key) = definition.node {
                crate::sema::header_attributes::bind_definition(
                    report,
                    facts,
                    DefId(u32::try_from(index).map_err(|_| {
                        Error::at(key.file, 0, "too many source header definitions")
                    })?),
                    key,
                )?;
            }
            if let Some(key) = definition.node {
                crate::sema::specialization::bind_trait(
                    report,
                    facts,
                    DefId(
                        u32::try_from(index).map_err(|_| {
                            Error::at(key.file, 0, "too many specialization traits")
                        })?,
                    ),
                    key,
                )?;
            }
            let Some(&contract) = definition.node.as_ref().and_then(|key| by_source.get(key))
            else {
                continue;
            };
            let key = definition
                .node
                .ok_or_else(|| Error::at(FileId(0), 0, "must_use declaration disappeared"))?;
            let unit = report.package.unit(key.file);
            let namespace = match unit.configured.ast.node(key.node).kind {
                Kind::Item(item)
                    if matches!(
                        unit.configured.ast.items[item as usize].kind,
                        ItemKind::Function { .. }
                    ) =>
                {
                    Namespace::Value
                }
                _ => Namespace::Type,
            };
            if definition.namespace != namespace {
                continue;
            }
            let definition = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::at(key.file, 0, "too many must_use definitions"))?,
            );
            let contracts = facts.must_use_contracts.get_or_insert_default();
            contracts.definitions.insert(definition, contract);
            contracts.work.declarations += 1;
        }
    }
    Ok(())
}

fn reason(
    source: &str,
    forest: &Forest,
    attribute: Attribute,
) -> Result<Option<Box<str>>, &'static str> {
    if attribute.unsafe_wrapper {
        return Err("must_use attribute cannot use an unsafe wrapper");
    }
    match attribute.input {
        Input::Empty => Ok(None),
        Input::Equals(range) if range.end == range.start + 1 => {
            let token = forest
                .token(range.start)
                .ok_or("must_use message requires a literal token")?;
            let Some(Value::String(bytes)) =
                literal::interpret(source, token).map_err(|error| error.message)?
            else {
                return Err("must_use message requires a string literal");
            };
            let reason = std::str::from_utf8(bytes.as_ref())
                .map_err(|_| "must_use message is not valid UTF-8")?;
            Ok(Some(reason.into()))
        }
        _ => Err("must_use attribute requires no arguments or one string literal message"),
    }
}
