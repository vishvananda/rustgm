//! Trait-parenthesis grammar is distinct from callable execution authority.
use crate::{
    expand::stream::Location,
    resolve::{bindings::FeatureName, expansion::{MacroAllowanceSession, Report}, model::{DefId, DefKind}, package::Error, unit::{FileId, NodeKey}},
    sema::model::Declarations,
    support::hash::HashMap,
    syntax::{ast::{Kind, NodeId}, attribute::Input, item::ItemKind, token_tree::Span},
};
#[derive(Clone, Copy)]
pub struct View {
    pub owner: NodeKey,
    pub declaration: NodeKey,
    pub span: Span,
}
struct Marker {
    view: View,
    effect: usize,
}
#[derive(Default)]
pub(in crate::sema) struct Contracts {
    markers: HashMap<DefId, Marker>,
    pub(in crate::sema) outputs: HashMap<DefId, DefId>,
}
impl Declarations {
    /// # Errors
    /// Rejects changed actual source owners and unknown imported marker contracts.
    pub fn parenthesis_marker(&self, report: &Report, definition: DefId) -> Result<Option<View>, Error> {
        if self.must_use_contracts.as_deref().is_some_and(|table| table.unknown.contains(&definition)) {
            return Err(Error::at(FileId(0), 0, "imported parenthesized trait marker transport is not implemented"));
        }
        let Some(marker) = self.must_use_contracts.as_deref().and_then(|table| table.parentheses.as_deref())
            .and_then(|table| table.markers.get(&definition)) else {return Ok(None)};
        let view = marker.view;
        let unit = report.package.unit(view.owner.file);
        let owner = unit.owner_indices[view.owner.node.get() as usize]
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1));
        if report.resolution.definitions.get(definition.0 as usize).is_none_or(|actual| actual.node != Some(view.declaration) || actual.kind != DefKind::Trait)
            || owner.is_none_or(|owner| !owner.enabled || owner.node != view.owner.node || !owner.attributes.contains(&marker.effect))
            || unit.retained_attributes(marker.effect..marker.effect+1).next().is_none_or(|(attribute, forest)| attribute.span != view.span || attribute.name(forest, unit.source.text()) != Some("rustc_paren_sugar")) {
            return Err(Error::at(view.owner.file, view.span.start, "parenthesized trait marker owner changed"));
        }
        Ok(Some(view))
    }
}
pub(super) fn collect(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let mut allowances = report.macro_allowances();
    let mut by_source = HashMap::default();
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {continue}
        let file = FileId(u32::try_from(file).map_err(|_| Error::at(FileId(0), 0, "too many trait marker files"))?);
        for owner in unit.configured.owners.iter().filter(|owner| owner.enabled) {
            for (effect, attribute, forest) in unit.retained_indexed_attributes(owner.attributes.clone()) {
                if attribute.name(forest, unit.source.text()) != Some("rustc_paren_sugar") {continue}
                let marker = marker(report, &mut allowances, NodeKey {file, node: owner.node}, effect, attribute, forest)?;
                let declaration = marker.view.declaration;
                if by_source.contains_key(&declaration) {return Err(Error::at(file, attribute.span.start, "duplicate rustc_paren_sugar attribute"))}
                if by_source.len() >= 1_000_000 {return Err(Error::at(file, attribute.span.start, "trait marker storage budget exceeded"))}
                by_source.insert(declaration, marker);
            }
        }
    }
    if by_source.is_empty() {return Ok(())}
    let table = facts.must_use_contracts.get_or_insert_default().parentheses.get_or_insert_default();
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if definition.kind != DefKind::Trait {continue}
        let Some(marker) = definition.node.and_then(|node| by_source.remove(&node)) else {continue};
        let definition = DefId(u32::try_from(index).map_err(|_| Error::at(marker.view.owner.file, marker.view.span.start, "too many marked trait definitions"))?);
        table.markers.insert(definition, marker);
    }
    if !by_source.is_empty() {return Err(Error::at(FileId(0), 0, "trait marker has no actual declaration identity"))}
    Ok(())
}
fn marker(report: &Report, allowances: &mut MacroAllowanceSession<'_>, owner: NodeKey, effect: usize,
    attribute: crate::syntax::attribute::Attribute, forest: &crate::syntax::token_tree::Forest) -> Result<Marker, Error> {
    let unit = report.package.unit(owner.file);
    let error = |message| Error::at(owner.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {return Err(error("rustc_paren_sugar requires a word attribute"))}
    let name = forest.tree(attribute.path.start).ok_or_else(|| error("missing trait marker name token"))?.span;
    let site = Location {file: owner.file, span: name};
    if !feature(report, allowances, site, false)? {return Err(error("rustc_paren_sugar requires rustc_attrs"))}
    if !feature(report, allowances, site, true)? {return Err(error("rustc_paren_sugar trait declarations require unboxed_closures"))}
    let mut node = owner.node;
    let mut remaining = unit.configured.ast.nodes.len();
    while let Kind::Attributed {value, ..} = unit.configured.ast.node(node).kind {
        remaining = remaining.checked_sub(1).ok_or_else(|| error("trait marker owner work budget exceeded"))?; node = value;
    }
    if !matches!(unit.configured.ast.node(node).kind, Kind::Item(item) if matches!(unit.configured.ast.items[item as usize].kind, ItemKind::Trait {..})) {return Err(error("rustc_paren_sugar requires a trait owner"))}
    Ok(Marker {view: View {owner, declaration: NodeKey {node, ..owner}, span: attribute.span}, effect})
}
pub(in crate::sema) fn feature(report: &Report, allowances: &mut MacroAllowanceSession<'_>, site: Location, unboxed: bool) -> Result<bool, Error> {
    if if unboxed {report.package.requested_unboxed_closures()}else{report.package.requested_internal_attributes()} {return Ok(true)}
    let name = FeatureName::new(if unboxed {"unboxed_closures"}else{"rustc_attrs"}).map_err(|message| Error::at(site.file, site.span.start, message))?;
    allowances.allows(site, &name)
}
pub(in crate::sema) fn name_site(report: &Report, key: NodeKey) -> Result<Location, Error> {
    let ast = &report.package.unit(key.file).configured.ast;
    let mut node: NodeId = key.node;
    for _ in 0..ast.nodes.len() {
        node = match ast.node(node).kind {
            Kind::Name(span, _) => return Ok(Location {file: key.file, span}),
            Kind::Applied {base, ..} => base,
            Kind::Path {segments, ..} => *ast.list(segments).last().ok_or_else(|| Error::at(key.file, 0, "empty parenthesized trait path"))?,
            _ => return Err(Error::at(key.file, ast.node(node).span.start, "parenthesized trait requires an actual name token")),
        };
    }
    Err(Error::at(key.file, 0, "parenthesized trait name work budget exceeded"))
}
