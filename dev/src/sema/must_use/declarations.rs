//! Validate retained syntax once, then bind its contract to real canonical items.
use super::{Contract, Declarations, Error, HashMap, HashSet, NodeKey, Report, WarningCode};
use crate::{
    literal::{self, Value},
    resolve::{
        lint_policy::producer::Producer,
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

/// State lasts for this one configured owner's existing attribute walk only.
#[derive(Default)]
struct OwnerFlags {
    coverage: bool,
    insignificant_dtor: bool,
}

pub(in crate::sema) fn collect(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    crate::sema::specialization::prepare(report, facts)?;
    let mut by_source = HashMap::default();
    let mut pointer_markers = HashMap::default();
    let mut mir_boundaries = crate::sema::inlining::MirBoundaries::default();
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
            let mut flags = OwnerFlags::default();
            for effect in owner.attributes.clone() {
                let Some((attribute, forest)) = unit.retained_attributes(effect..effect + 1).next()
                else {
                    continue;
                };
                if unit.derive_is_expanded(effect) {
                    report.expanded_derive(owner_key, effect)?;
                    continue;
                }
                if matches!(
                    attribute.name(forest, unit.source.text()),
                    Some(
                        "rustc_no_mir_inline"
                            | "rustc_no_writable"
                            | "rustc_should_not_be_called_on_const_items"
                    )
                ) {
                    mir_boundaries.consume(
                        report,
                        facts,
                        (owner_key, key),
                        (attribute, effect),
                        forest,
                        &mut allowances,
                    )?;
                    continue;
                }
                if consume_annotation(
                    report,
                    facts,
                    (owner_key, key),
                    (attribute, effect),
                    forest,
                    &mut allowances,
                    &mut flags,
                )? {
                    if attribute.name(forest, unit.source.text()) == Some("rustc_as_ptr") {
                        retain_pointer_marker(facts, key, &mut pointer_markers)?;
                    }
                    continue;
                }
                consume_must_use(
                    report,
                    facts,
                    (owner_key, key, supported),
                    (attribute, effect),
                    forest,
                    &mut by_source,
                )?;
            }
        }
    }
    bind_definitions(
        report,
        facts,
        &by_source,
        &mut pointer_markers,
        &mut mir_boundaries,
    )?;
    crate::sema::specialization::finish(facts)?;
    complete_imports(report, facts)
}
fn consume_must_use(
    report: &Report,
    facts: &mut Declarations,
    (owner, declaration, supported): (NodeKey, NodeKey, bool),
    (attribute, effect): (Attribute, usize),
    forest: &Forest,
    by_source: &mut HashMap<NodeKey, usize>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(owner, attribute.span)?;
    let unit = report.package.unit(declaration.file);
    let reason = reason(unit.source.text(), forest, attribute)
        .map_err(|message| Error::at(declaration.file, attribute.span.start, message))?;
    let message = if !supported {
        Some("must_use attribute requires a function, nominal type or trait")
    } else if by_source.contains_key(&declaration) {
        Some("duplicate must_use attribute")
    } else {
        None
    };
    if let Some(message) = message {
        facts.must_use_diagnostic(
            report,
            owner,
            attribute.span.start,
            0,
            WarningCode::UnusedAttributes,
            message.into(),
        )?;
        return Ok(());
    }
    let index = retain(
        facts,
        Contract {
            image: std::sync::Arc::clone(unit.source.original_image()),
            owner,
            declaration,
            span: attribute.span,
            effect,
            reason,
        },
    )?;
    by_source.insert(declaration, index);
    Ok(())
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
    flags: &mut OwnerFlags,
) -> Result<bool, Error> {
    let consumer = attribute
        .name(forest, report.package.unit(declaration.file).source.text())
        .and_then(Consumer::named);
    match consumer {
        Some(Consumer::ColdAdvisory) => {
            crate::sema::inlining::consume_cold(report, facts, declaration, attribute)?;
        }
        Some(Consumer::PointerLintMarker) => {
            crate::sema::header_attributes::consume_pointer_lint(
                report,
                facts,
                declaration,
                attribute,
                forest,
                allowances,
            )?;
        }
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
            if flags.coverage {
                return Err(Error::at(
                    declaration.file,
                    attribute.span.start,
                    "duplicate coverage attribute",
                ));
            }
            flags.coverage = true;
            crate::sema::coverage_annotations::consume(
                report,
                facts,
                declaration,
                attribute,
                forest,
                allowances,
            )?;
        }
        Some(Consumer::InsignificantDestructor) => {
            crate::sema::insignificant_drop::consume(
                report,
                facts,
                declaration,
                attribute,
                forest,
                allowances,
                &mut flags.insignificant_dtor,
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
        Some(Consumer::LegacyConstGenerics) => {
            crate::sema::header_attributes::consume_legacy_indices(
                report,
                facts,
                (owner, declaration),
                (attribute, effect),
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
        Some(Consumer::IncoherentTarget) => {
            crate::sema::header_attributes::consume_incoherent_target(
                report,
                facts,
                (owner, declaration),
                (attribute, effect),
            )?;
        }
        Some(Consumer::ReservationImpl) => {
            crate::sema::associated::reserved_impls::consume(
                report,
                facts,
                owner,
                declaration,
                attribute,
                effect,
            )?;
        }
        Some(Consumer::NonConstTraitMethod) => {
            crate::sema::header_attributes::consume_nonconst_member(
                report,
                facts,
                owner,
                declaration,
                attribute,
                effect,
            )?;
        }
        Some(Consumer::Comptime) => {
            crate::sema::header_attributes::consume_comptime(
                report,
                facts,
                owner,
                declaration,
                attribute,
                effect,
            )?;
        }
        Some(Consumer::Promotion) => {
            crate::sema::header_attributes::consume_promotable(
                report,
                facts,
                owner,
                declaration,
                attribute,
                effect,
            )?;
        }
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
/// One sparse draft is consumed by the existing definition walk, including
/// repeated physical source rows. Every actual foreign owner is checked.
fn retain_pointer_marker(
    facts: &mut Declarations,
    source: NodeKey,
    markers: &mut HashMap<NodeKey, bool>,
) -> Result<(), Error> {
    use crate::support::retained::Growth;
    crate::sema::header_attributes::reserve_source_contracts(facts, 1, 0)?;
    if markers.contains_key(&source) {
        return Ok(());
    }
    let growth = Growth::map::<NodeKey, bool>(markers.len(), markers.capacity(), 1);
    crate::sema::header_attributes::reserve_source_contracts(facts, growth.work, growth.bytes)?;
    markers
        .try_reserve(1)
        .map_err(|_| Error::unlocated("pointer marker draft allocation failed"))?;
    markers.insert(source, false);
    Ok(())
}
fn bind_pointer_marker(
    report: &Report,
    index: usize,
    definition: &crate::resolve::model::Definition,
    markers: &mut HashMap<NodeKey, bool>,
) -> Result<(), Error> {
    let Some(key) = definition.node else {
        return Ok(());
    };
    let Some(bound) = markers.get_mut(&key) else {
        return Ok(());
    };
    let id = DefId(
        u32::try_from(index).map_err(|_| Error::unlocated("too many pointer marker owners"))?,
    );
    if definition.kind != crate::resolve::model::DefKind::Function
        || definition.namespace != Namespace::Value
        || report.resolution.foreign_owners.contains_key(&id)
    {
        return Err(Error::at(
            key.file,
            0,
            "rustc_as_ptr requires its actual non-foreign function or method owner",
        ));
    }
    *bound = true;
    Ok(())
}

fn bind_macro_helpers(
    report: &Report,
    facts: &mut Declarations,
    index: usize,
    definition: &crate::resolve::model::Definition,
) -> Result<(), Error> {
    if definition.kind == crate::resolve::model::DefKind::Macro && definition.node.is_some() {
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
    Ok(())
}

fn bind_definitions(
    report: &Report,
    facts: &mut Declarations,
    by_source: &HashMap<NodeKey, usize>,
    pointer_markers: &mut HashMap<NodeKey, bool>,
    mir_boundaries: &mut crate::sema::inlining::MirBoundaries,
) -> Result<(), Error> {
    mir_boundaries.prepare_binding(report, facts)?;
    if !pointer_markers.is_empty()
        || report
            .lint_producer()
            .is_some_and(Producer::has_impl_annotations)
    {
        crate::sema::header_attributes::reserve_source_contracts(
            facts,
            report
                .resolution
                .definitions
                .len()
                .saturating_add(pointer_markers.len()),
            0,
        )?;
    }
    if !by_source.is_empty()
        || !pointer_markers.is_empty()
        || !mir_boundaries.is_empty()
        || report
            .lint_producer()
            .is_some_and(Producer::has_impl_annotations)
        || !facts.associated.owners().is_empty()
        || report.package.source_core().is_some()
        || crate::sema::header_attributes::has_comptime_headers(facts)
        || facts
            .must_use_contracts
            .as_deref()
            .is_some_and(|owner| owner.specialization.is_some())
    {
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            bind_pointer_marker(report, index, definition, pointer_markers)?;
            mir_boundaries.bind(report, facts, index, definition)?;
            crate::sema::associated::reserved_impls::bind(report, facts, index, definition)?;
            bind_macro_helpers(report, facts, index, definition)?;
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
    if pointer_markers.values().any(|bound| !bound) {
        return Err(Error::unlocated(
            "pointer lint marker lacks its actual function definition",
        ));
    }
    mir_boundaries.finish()?;
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
