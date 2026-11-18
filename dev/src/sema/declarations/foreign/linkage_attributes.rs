//! Retained linkage attributes decode literal tokens once into the owning exact-name pool.
#[cfg(test)]
#[path = "../../../../tests/invariants/linkage_editions.rs"]
mod tests;
use crate::support::hash::HashMap;
use crate::{
    literal::{self, Value},
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        package::{Error, ModuleId},
        unit::{FileId, NodeKey},
    },
    sema::foreign::Registry,
    support::linkage::SymbolNameId,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::ItemKind,
        token_tree::Forest,
    },
};
#[derive(Clone, Copy, Default)]
pub(in crate::sema::declarations) struct Attributes {
    pub link: Option<SymbolNameId>,
    pub export: Option<SymbolNameId>,
    pub no_mangle: bool,
    /// `#[rustc_std_internal_symbol]`: the item links under rustc's
    /// version-mangled internal name. Its own consumer validates the form.
    pub internal: bool,
    pub extern_weak: Option<u32>,
}
pub(in crate::sema::declarations) fn collect(
    report: &Report,
    facts: &mut crate::sema::model::Declarations,
) -> Result<HashMap<NodeKey, Attributes>, Error> {
    let mut sources = HashMap::<NodeKey, Attributes>::default();
    let mut permissions = super::llvm_permissions::Draft::default();
    let mut allowances = report.macro_allowances();
    let mut weak_requests = HashMap::default();
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(file)
                .map_err(|_| Error::at(FileId(0), 0, "too many linkage source files"))?,
        );
        for owner in &unit.configured.owners {
            if !owner.enabled || owner.attributes.is_empty() {
                continue;
            }
            let mut node = owner.node;
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                node = value;
            }
            let declaration = NodeKey { file, node };
            for (retained, attribute, forest) in
                unit.retained_indexed_attributes(owner.attributes.clone())
            {
                let Some(name) = attribute.name(forest, unit.source.text()) else {
                    continue;
                };
                if name == "rustc_std_internal_symbol" {
                    sources.entry(declaration).or_default().internal = true;
                    continue;
                }
                if crate::support::decorations::Consumer::named(name)
                    != Some(crate::support::decorations::Consumer::Linkage)
                {
                    continue;
                }
                let error = |message| Error::at(file, attribute.span.start, message);
                let item = linkage_item(unit, node).map_err(error)?;
                if name == "linkage" {
                    if !matches!(item, ItemKind::Static { .. }) {
                        return Err(error("extern_weak currently requires a foreign static"));
                    }
                    let site = (
                        NodeKey {
                            file,
                            node: owner.node,
                        },
                        retained,
                        attribute,
                    );
                    retain_weak(
                        report,
                        facts,
                        site,
                        forest,
                        &mut weak_requests,
                        &mut allowances,
                    )?;
                    let selected = sources.entry(declaration).or_default();
                    if selected.extern_weak.replace(attribute.span.start).is_some() {
                        return Err(error("duplicate extern_weak linkage attribute"));
                    }
                    continue;
                }
                safety(report, file, forest, attribute, name)?;
                let selected = sources.entry(declaration).or_default();
                if name == "no_mangle" {
                    if attribute.input != Input::Empty {
                        return Err(error("no_mangle requires no arguments"));
                    }
                    selected.no_mangle = true;
                } else {
                    let id = exact(unit.source.text(), forest, attribute, &mut facts.foreign)
                        .map_err(error)?;
                    if name == "link_name"
                        && selected.link.is_none()
                        && facts
                            .foreign
                            .as_deref()
                            .and_then(|registry| registry.names().llvm(id))
                            .is_some()
                    {
                        permissions.require(report, facts, declaration, &mut allowances)?;
                    }
                    let slot = if name == "link_name" {
                        &mut selected.link
                    } else {
                        &mut selected.export
                    };
                    slot.get_or_insert(id);
                }
            }
        }
    }
    Ok(sources)
}
fn linkage_item(
    unit: &crate::resolve::unit::Unit,
    node: crate::syntax::ast::NodeId,
) -> Result<ItemKind, &'static str> {
    const MESSAGE: &str = "linkage attribute requires a function or static item";
    let Kind::Item(index) = unit.configured.ast.node(node).kind else {
        return Err(MESSAGE);
    };
    let kind = unit.configured.ast.items[index as usize].kind;
    if !matches!(kind, ItemKind::Function { .. } | ItemKind::Static { .. }) {
        return Err(MESSAGE);
    }
    Ok(kind)
}

fn retain_weak(
    report: &Report,
    facts: &mut crate::sema::model::Declarations,
    (owner, retained, attribute): (NodeKey, usize, Attribute),
    forest: &Forest,
    requests: &mut HashMap<ModuleId, bool>,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    let error = |message| Error::at(owner.file, attribute.span.start, message);
    if report.package.source_core().is_none() {
        return Err(error(
            "extern_weak linkage requires the actual Source provider",
        ));
    }
    if attribute.unsafe_wrapper || attribute.style != Style::Outer {
        return Err(error("linkage requires an unwrapped outer attribute"));
    }
    facts.reserve_must_use_source(owner, attribute.span)?;
    let root = report.package.file_crate_root(owner.file);
    let requested = if let Some(&requested) = requests.get(&root) {
        requested
    } else {
        let requested = report.package.requested_external_linkage(owner.file)?;
        let growth = crate::support::retained::Growth::map::<ModuleId, bool>(
            requests.len(),
            requests.capacity(),
            1,
        );
        crate::sema::header_attributes::reserve_source_contracts(facts, growth.work, growth.bytes)?;
        requests
            .try_reserve(1)
            .map_err(|_| error("linkage feature cache allocation failed"))?;
        requests.insert(root, requested);
        requested
    };
    if !requested
        && !allowances.allows_attribute(
            owner,
            retained,
            attribute,
            &FeatureName::new("linkage").map_err(error)?,
        )?
    {
        return Err(error("extern_weak requires this crate's linkage feature"));
    }
    let unit = report.package.unit(owner.file);
    if linkage_text(unit.source.text(), forest, attribute)
        .map_err(error)?
        .as_ref()
        != b"extern_weak"
    {
        return Err(error(
            "source linkage currently supports only extern_weak foreign statics",
        ));
    }
    crate::sema::header_attributes::reserve_source_contracts(
        facts,
        2,
        4 * size_of::<(NodeKey, Attributes)>(),
    )?;
    Ok(())
}
fn safety(
    report: &Report,
    file: FileId,
    forest: &Forest,
    attribute: Attribute,
    name: &str,
) -> Result<(), Error> {
    if attribute.unsafe_wrapper == (name != "link_name") {
        return Ok(());
    }
    let error = |message| Error::at(file, attribute.span.start, message);
    if !attribute.unsafe_wrapper && name != "link_name" {
        // The name owns the edition; a macro may supply the surrounding #[...]
        // from another edition. Retain the parsed token's exact source ancestry.
        let token = forest
            .token(attribute.path.start)
            .ok_or_else(|| error("missing linkage attribute name token"))?;
        let edition = crate::resolve::unit::editions::token(file, token, |file| {
            report.package.units.get(file.0 as usize)
        })
        .map_err(|failure| error(failure.message))?;
        if edition.before_2024() {
            return Ok(());
        }
    }
    Err(error("invalid unsafe wrapper for linkage attribute"))
}
fn exact(
    source: &str,
    forest: &Forest,
    attribute: Attribute,
    registry: &mut Option<Box<Registry>>,
) -> Result<SymbolNameId, &'static str> {
    let bytes = linkage_text(source, forest, attribute)?;
    let spelling =
        std::str::from_utf8(bytes.as_ref()).map_err(|_| "linkage attribute is not valid UTF-8")?;
    registry
        .get_or_insert_default()
        .names_mut()
        .intern(spelling)
}
fn linkage_text<'source>(
    source: &'source str,
    forest: &Forest,
    attribute: Attribute,
) -> Result<std::borrow::Cow<'source, [u8]>, &'static str> {
    let Input::Equals(range) = attribute.input else {
        return Err("linkage attribute requires a string literal");
    };
    if range.end != range.start + 1 {
        return Err("linkage attribute requires one string literal");
    }
    let token = forest
        .token(range.start)
        .ok_or("linkage attribute requires a literal token")?;
    let Some(Value::String(bytes)) = literal::interpret(source, token).map_err(|e| e.message)?
    else {
        return Err("linkage attribute requires a string literal");
    };
    Ok(bytes)
}
