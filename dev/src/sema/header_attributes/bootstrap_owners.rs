//! Legacy compiler metadata retains ownership without minting source-header facts.
use super::{DefId, Error, Report};
use crate::{
    resolve::model::{Access, DefKind},
    support::{comparisons::Comparison, language_hooks::Hook, library_types::LibraryType},
};

pub(in crate::sema) fn owner(report: &Report, definition: DefId) -> Result<bool, Error> {
    if report.package.uses_source_bindings() {
        return Ok(false);
    }
    let Some(kind) = registered_kind(report, definition) else {
        return Ok(false);
    };
    let actual = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .ok_or_else(|| Error::unlocated("bootstrap diagnostic owner disappeared"))?;
    if actual.node.is_some()
        || actual.kind != kind
        || actual.access != Access::Public
        || report
            .resolution
            .bindings
            .get(actual.scope, actual.name, actual.namespace)
            .is_none_or(|binding| binding.target != definition)
    {
        return Err(Error::unlocated("bootstrap diagnostic owner changed"));
    }
    if let Some(imported) = &report.resolution.crates
        && let Some(catalog) = imported.origin_catalog().map_err(Error::unlocated)?
        && catalog
            .declaration_if_imported(definition)
            .map_err(Error::unlocated)?
            .is_some()
    {
        return Err(Error::unlocated(
            "bootstrap diagnostic owner became imported",
        ));
    }
    Ok(true)
}

fn registered_kind(report: &Report, definition: DefId) -> Option<DefKind> {
    let resolution = &report.resolution;
    if resolution.drop_trait == Some(definition)
        || resolution.builtin_traits.identify(definition).is_some()
    {
        return Some(DefKind::BuiltinTrait);
    }
    if resolution.intrinsics.identify(definition).is_some()
        || resolution
            .library_types
            .slice_length()
            .is_some_and(|entry| entry.definition == definition)
        || trait_function(report, definition)
    {
        return Some(DefKind::Function);
    }
    for kind in LibraryType::ALL {
        let Some(entry) = resolution.library_types.get(kind) else {
            continue;
        };
        if entry.definition == definition {
            return Some(if kind.is_struct() {
                DefKind::Struct
            } else {
                DefKind::Enum
            });
        }
        if entry.variants.iter().any(|variant| {
            variant.constructor == Some(definition) || variant.type_definition == Some(definition)
        }) {
            return Some(if kind.is_struct() {
                DefKind::Constructor
            } else {
                DefKind::Variant
            });
        }
    }
    None
}

/// These inventories have fixed language-defined widths, independent of source size.
fn trait_function(report: &Report, definition: DefId) -> bool {
    let traits = &report.resolution.builtin_traits;
    Hook::ALL.into_iter().any(|kind| {
        traits
            .language(kind)
            .is_some_and(|entry| entry.method == definition)
    }) || Comparison::ALL.into_iter().any(|kind| {
        traits
            .comparison(kind)
            .and_then(|entry| entry.methods)
            .is_some_and(|methods| methods.iter().any(|(_, method)| method == definition))
    }) || traits.cloning().is_some_and(|entry| {
        entry.method == definition
            || entry
                .implementation
                .is_some_and(|implementation| implementation.method == definition)
    }) || traits.iteration().is_some_and(|entry| {
        entry
            .declarations
            .iter()
            .any(|declaration| declaration.method == definition)
            || entry
                .blankets
                .iter()
                .any(|implementation| implementation.method == definition)
    }) || traits.collections().is_some_and(|entries| {
        entries
            .declarations()
            .any(|entry| entry.method == definition)
    }) || traits.ranges().is_some_and(|entries| {
        entries
            .implementations
            .iter()
            .any(|entry| entry.method == definition)
    })
}
