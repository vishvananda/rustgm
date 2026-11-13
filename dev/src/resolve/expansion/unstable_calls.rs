//! An imported definition row's `#[unstable]` feature gates every call of it,
//! through any path or re-export, and every single `use` import that binds
//! it, called or not. The crate must request the feature, or the call or
//! import must sit in an expansion whose definition may use it, as rustc
//! checks macro stability at the call's own span and at every `use` path.
use super::{
    Body, Builtin, Definition, Error, Invocation, Location, MacroAllowanceSession, Report,
};
use crate::{
    resolve::{
        bindings::FeatureName,
        crates::Imported,
        model::{DefId, Namespace, Origin},
        unit::NodeKey,
    },
    support::{crates::Interface, hash::HashMap},
};

/// The library requests and the expansion allowances one validation pass
/// consults; the allowance session is opened only for a gated site.
struct Gate<'a> {
    report: &'a Report,
    session: Option<MacroAllowanceSession<'a>>,
    steps: usize,
    source_names: HashMap<NodeKey, HashMap<u32, Location>>,
}

impl Gate<'_> {
    fn check(
        &mut self,
        site: Location,
        feature: &str,
        source_list: Option<&Invocation>,
    ) -> Result<(), Error> {
        let error = |message| Error::at(site.file, site.span.start, message);
        let name = FeatureName::new(feature).map_err(error)?;
        let requested = self
            .report
            .package
            .library_policy()
            .map(|library| library.requested_feature(&name, &mut self.steps))
            .transpose()
            .map_err(error)?
            .unwrap_or(false);
        if requested {
            return Ok(());
        }
        let site = if let Some(invocation) = source_list {
            self.derive_name(invocation)?
        } else {
            site
        };
        if self
            .session
            .get_or_insert_with(|| self.report.macro_allowances())
            .allows(site, &name)?
        {
            return Ok(());
        }
        Err(Error::at(
            site.file,
            site.span.start,
            format!("use of unstable library macro requires feature `{feature}`"),
        ))
    }

    /// Index a configured owner's real name tokens once. A literal `#[` cannot
    /// lend its definition's allowance to a captured attribute path.
    fn derive_name(&mut self, invocation: &Invocation) -> Result<Location, Error> {
        let owner = invocation.node;
        let error = |message| Error::at(owner.file, invocation.original.start, message);
        if !self.source_names.contains_key(&owner) {
            let unit = self.report.package.unit(owner.file);
            let configured = unit
                .owner_indices
                .get(owner.node.get() as usize)
                .copied()
                .flatten()
                .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
                .ok_or_else(|| error("source derive lost its configured attribute owner"))?;
            let count = configured.attributes.len();
            let owners = if self.source_names.len() < self.source_names.capacity() {
                0
            } else {
                crate::support::retained::new_map::<NodeKey, HashMap<u32, Location>>(
                    self.source_names.len().saturating_add(1),
                )
                .saturating_sub(crate::support::retained::map(&self.source_names))
            };
            let growth =
                owners.saturating_add(crate::support::retained::new_map::<u32, Location>(count));
            self.report
                .package
                .source_contract_payer()
                .map_err(error)?
                .ok_or_else(|| error("source derive API lookup lacks its actual payer"))?
                .charge(8_usize.saturating_add(4 * count), growth)
                .map_err(error)?;
            let mut names = HashMap::default();
            names
                .try_reserve(count)
                .map_err(|_| error("source derive name index allocation failed"))?;
            self.source_names
                .try_reserve(1)
                .map_err(|_| error("source derive owner index allocation failed"))?;
            for index in configured.attributes.clone() {
                let effect = &unit.configured.effects[index];
                let attribute = effect.attribute.attribute;
                let forest = &unit.configured.ast.attributes[effect.capture as usize].forest;
                let token = attribute
                    .path
                    .end
                    .checked_sub(1)
                    .and_then(|index| forest.tree(index))
                    .ok_or_else(|| error("source derive attribute lost its actual name token"))?;
                if names
                    .insert(
                        attribute.span.start,
                        Location {
                            file: owner.file,
                            span: token.span,
                        },
                    )
                    .is_some()
                {
                    return Err(error("source derive owner repeats an attribute identity"));
                }
            }
            self.source_names.insert(owner, names);
        }
        self.source_names
            .get(&owner)
            .and_then(|names| names.get(&invocation.original.start))
            .copied()
            .ok_or_else(|| error("source derive call differs from its actual attribute"))
    }
}

/// Check each call of an imported row, then each single import binding one,
/// once the library requests are complete.
/// # Errors
/// Rejects an unrequested, unallowed feature by name, and a changed row.
pub(in crate::resolve) fn validate(report: &Report) -> Result<(), Error> {
    let mut gate = Gate {
        report,
        session: None,
        steps: 0,
        source_names: HashMap::default(),
    };
    for invocation in &report.invocations {
        let definition = report
            .definitions
            .get(invocation.definition)
            .ok_or_else(|| Error::unlocated("source derive names an unknown macro"))?;
        if let Some(feature) = local_derive_feature(report, definition)? {
            gate.check(
                Location {
                    file: invocation.node.file,
                    span: invocation.original,
                },
                feature,
                Some(invocation),
            )?;
        }
    }
    let Some(crates) = report.resolution.crates.as_deref() else {
        return Ok(());
    };
    let artifacts = crates.dependencies().artifacts();
    for invocation in &report.invocations {
        let site = Location {
            file: invocation.node.file,
            span: invocation.original,
        };
        let error = |message| Error::at(site.file, site.span.start, message);
        report
            .definitions
            .get(invocation.definition)
            .ok_or_else(|| error("macro call names an unknown definition"))?;
        let Some(imported) =
            super::transported_macros::imported(&report.resolution, invocation.definition)
        else {
            continue;
        };
        let feature = artifacts
            .get(imported.artifact)
            .and_then(Interface::macros)
            .and_then(|macros| macros.unstable(imported.row))
            .ok_or_else(|| error("imported macro stability row disappeared"))?;
        if let Some(feature) = feature {
            gate.check(site, feature, None)?;
        }
    }
    validate_imports(crates, &mut gate)
}

/// Consuming an attribute list must retain the selected source macro's API gate.
/// Read its completed policy by canonical identity; spelling supplies no feature.
fn local_derive_feature<'a>(
    report: &'a Report,
    definition: &Definition,
) -> Result<Option<&'a str>, Error> {
    if !matches!(
        definition.body,
        Body::Builtin(Builtin::Derive | Builtin::DeriveConst)
    ) {
        return Ok(None);
    }
    let source = definition
        .node
        .ok_or_else(|| Error::unlocated("source derive lacks its actual declaration"))?;
    let error = |message| Error::at(source.file, 0, message);
    if let Some(payer) = report.package.source_contract_payer().map_err(error)? {
        payer.charge(8, 0).map_err(error)?;
    }
    let id = report
        .resolution
        .item_definitions
        .get(&(definition.module, source.node, Namespace::Macro))
        .ok_or_else(|| error("source derive lacks its canonical macro identity"))?;
    let actual = report
        .resolution
        .definitions
        .get(id.0 as usize)
        .ok_or_else(|| error("source derive lost its canonical definition"))?;
    if actual.node != Some(source) || actual.name != definition.name {
        return Err(error("source derive changed its canonical declaration"));
    }
    let Some(library) = report
        .package
        .library_policy()
        .filter(|library| library.is_staged())
    else {
        return Ok(None);
    };
    let policy = library
        .inventory()
        .ok_or_else(|| error("source derive lacks its completed API inventory"))?
        .local_declaration(*id)
        .map_err(error)?;
    Ok(policy
        .filter(|policy| {
            matches!(
                policy.level,
                crate::resolve::stability::Level::Unstable { .. }
            )
        })
        .map(|policy| policy.feature.as_ref()))
}

/// A single `use` import of an unstable row needs its feature even when the
/// crate never calls it. A glob import names only its module, whose own
/// stability is checked as a path, so its macro bindings are not gated here.
fn validate_imports(crates: &Imported, gate: &mut Gate<'_>) -> Result<(), Error> {
    let artifacts = crates.dependencies().artifacts();
    let mut gated: HashMap<DefId, &str> = HashMap::default();
    for row in crates.macros() {
        let feature = artifacts
            .get(row.artifact)
            .and_then(Interface::macros)
            .and_then(|macros| macros.unstable(row.row))
            .ok_or_else(|| Error::unlocated("imported macro stability row disappeared"))?;
        if let Some(feature) = feature {
            gated.insert(row.definition, feature);
        }
    }
    if gated.is_empty() {
        return Ok(());
    }
    let report = gate.report;
    for (_, namespace, binding) in report.resolution.bindings.all_bindings() {
        let (Namespace::Macro, Origin::Import(index)) = (namespace, binding.origin) else {
            continue;
        };
        let Some(&feature) = gated.get(&binding.target) else {
            continue;
        };
        let node = report
            .resolution
            .imports
            .get(index as usize)
            .ok_or_else(|| Error::unlocated("macro import binding names an unknown import"))?
            .node;
        let span = report
            .package
            .unit(node.file)
            .configured
            .ast
            .node(node.node)
            .span;
        gate.check(
            Location {
                file: node.file,
                span,
            },
            feature,
            None,
        )?;
    }
    Ok(())
}
