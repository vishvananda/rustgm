//! One imported owner maps immutable availability; each source use retains its binding route.
mod path_compatibility;
use crate::{
    expand::marks::ItemContext,
    resolve::{
        body_origin::{Catalog, ImportedDeclarationId},
        crates::Imported,
        expansion::Report,
        lookup::{BindingKey, Observations, Selection},
        model::{DefId, DefKind, Namespace, Resolution, Target},
        names::Names,
        package::{Error, Package},
        unit::NodeKey,
    },
    support::{
        crates::{
            Dependencies, MAX_BYTES, MAX_COMPONENTS, generics::source as s,
            owner_catalog::DeclarationRef,
        },
        hash::{HashMap, HashSet},
        retained,
    },
};
pub(in crate::resolve) use path_compatibility::PathKind;
pub(crate) use path_compatibility::visit_warnings;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

impl crate::resolve::index::Resolver<'_> {
    pub(in crate::resolve) fn availability_trace(
        &self,
        length: usize,
    ) -> Result<Option<Observations>, Error> {
        self.facts
            .crates
            .as_deref()
            .and_then(|imports| imports.availability.as_deref())
            .map(|imports| imports.trace(length))
            .transpose()
    }
    pub(in crate::resolve) fn record_availability(
        &mut self,
        site: NodeKey,
        trace: Option<&Observations>,
        target: Target,
    ) -> Result<(), Error> {
        let selected_target = match target {
            Target::Definition(id) => Some((id, PathKind::Body(self.facts.references.len()))),
            Target::Associated(id) => Some((id, PathKind::Associated(self.facts.references.len()))),
            _ => None,
        };
        self.record_path_availability(site, trace, selected_target)
    }
    pub(in crate::resolve) fn record_path_availability(
        &mut self,
        site: NodeKey,
        trace: Option<&Observations>,
        target: Option<(DefId, PathKind)>,
    ) -> Result<(), Error> {
        if let Some((terminal, kind)) = target
            && let Some(selected) = trace.and_then(|trace| trace.selected.as_deref())
            && let Some(imports) = self
                .facts
                .crates
                .as_deref_mut()
                .and_then(|imports| imports.availability.as_deref_mut())
        {
            imports.record(site, terminal, kind, selected)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Declaration {
    artifact: usize,
    reference: DeclarationRef,
    handle: ImportedDeclarationId,
    kind: DefKind,
}
#[derive(Clone, Copy)]
struct Binding {
    artifact: usize,
    index: usize,
    target: DefId,
    source: s::BindingSource,
}
#[derive(Clone, Copy)]
struct PathEntry {
    selected: crate::resolve::model::Binding,
    key: BindingKey,
    binding: Binding,
}
#[derive(Clone, Copy, Default)]
struct Budget {
    steps: usize,
    bytes: usize,
}
impl Budget {
    fn charge(&mut self, steps: usize, bytes: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(steps);
        self.bytes = self.bytes.saturating_add(bytes);
        if self.steps > MAX_COMPONENTS || self.bytes > MAX_BYTES {
            return Err(Error::unlocated(
                "imported availability work or storage budget exceeded",
            ));
        }
        Ok(())
    }
}
pub(in crate::resolve) struct Imports {
    catalog: Rc<Catalog>,
    inventory: Arc<Dependencies>,
    declarations: HashMap<DefId, Option<Declaration>>,
    bindings: HashMap<BindingKey, Binding>,
    features: HashSet<String>,
    entries: Vec<PathEntry>,
    paths: Vec<path_compatibility::PathUse>,
    paths_complete: bool,
    /// Denied uses inside expansions, awaiting their frames' allowance lists.
    deferred: RefCell<Vec<Deferred>>,
    budget: Cell<Budget>,
}
/// A use of an unstable imported API whose site lies in a macro expansion: an
/// imported or local definition's `allow_internal_unstable` list may admit it.
struct Deferred {
    site: NodeKey,
    feature: String,
}
impl Imports {
    fn charge(&self, steps: usize, bytes: usize) -> Result<(), Error> {
        let mut budget = self.budget.get();
        let result = budget.charge(steps, bytes);
        self.budget.set(budget);
        result
    }
    fn same_owner(&self, imported: &Imported) -> Result<(), Error> {
        self.charge(2, 0)?;
        let catalog = imported
            .origin_catalog()
            .map_err(Error::unlocated)?
            .ok_or_else(|| Error::unlocated("availability catalog is absent"))?;
        if !Rc::ptr_eq(&self.catalog, &catalog)
            || !Arc::ptr_eq(&self.inventory, &imported.shared_dependencies())
        {
            return Err(Error::unlocated(
                "availability belongs to another imported catalog or inventory",
            ));
        }
        Ok(())
    }
    fn rows(&self, artifact: usize) -> Result<&s::AvailabilityRows, Error> {
        self.charge(1, 0)?;
        self.inventory
            .artifacts()
            .get(artifact)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .ok_or_else(|| Error::unlocated("availability artifact descriptor is absent"))?
            .availability()
            .map_err(Error::unlocated)
    }
    pub(in crate::resolve) fn has_feature(&self, name: &str) -> Result<bool, Error> {
        self.charge(name.len().saturating_add(1), 0)?;
        Ok(self.features.contains(name))
    }
    fn policy_allowed(
        &self,
        package: &Package,
        _site: NodeKey,
        artifact: usize,
        reference: Option<s::AvailabilityRef>,
    ) -> Result<bool, Error> {
        let Some(reference) = reference else {
            return Ok(true);
        };
        let policy = self
            .rows(artifact)?
            .policies
            .get(reference.0 as usize)
            .ok_or_else(|| Error::unlocated("availability use has a foreign policy index"))?;
        self.charge(policy.feature.len().saturating_mul(4).saturating_add(1), 0)?;
        if matches!(
            policy.level,
            s::AvailabilityLevel::Stable(_) | s::AvailabilityLevel::StableWithCompatibility { .. }
        ) {
            return Ok(true);
        }
        let normalized;
        let spelling = if policy.feature.is_ascii() {
            policy.feature.as_str()
        } else {
            normalized = Names::normalize_paid(&policy.feature, &mut |growth| {
                self.charge(growth.work, growth.bytes)
                    .map_err(|_| "availability normalization budget exhausted")
            })
            .map_err(Error::unlocated)?;
            &normalized
        };
        let feature =
            crate::resolve::bindings::FeatureName::new(spelling).map_err(Error::unlocated)?;
        let requested = if let Some(library) = package.library_policy() {
            let mut steps = self.budget.get().steps;
            let result = library.requested_feature(&feature, &mut steps);
            let old = self.budget.get().steps;
            self.charge(steps.saturating_sub(old), 0)?;
            result.map_err(Error::unlocated)?
        } else {
            false
        };
        Ok(requested)
    }
    fn policy(
        &self,
        package: &Package,
        site: NodeKey,
        artifact: usize,
        reference: Option<s::AvailabilityRef>,
    ) -> Result<(), Error> {
        if self.policy_allowed(package, site, artifact, reference)? {
            return Ok(());
        }
        let reference =
            reference.ok_or_else(|| Error::unlocated("denied policy has no actual row"))?;
        let feature = &self.rows(artifact)?.policies[reference.0 as usize].feature;
        let start = package
            .unit(site.file)
            .configured
            .ast
            .node(site.node)
            .span
            .start;
        let context = package.syntax_context(site.file, start)?;
        if package
            .marks
            .outer(context)
            .map_err(Error::unlocated)?
            .is_some()
        {
            return self.defer(site, feature);
        }
        Err(Error::at(
            site.file,
            start,
            format!("use of unstable library API requires feature `{feature}`"),
        ))
    }
    /// Retain one denied use inside an expansion until its frames' allowance
    /// lists are known. It owns its spelling, so it pays for those bytes.
    fn defer(&self, site: NodeKey, written: &str) -> Result<(), Error> {
        let feature = self.spelling(written)?.into_owned();
        let growth = retained::Growth::vector::<Deferred>(
            self.deferred.borrow().len(),
            self.deferred.borrow().capacity(),
            1,
        );
        self.charge(
            growth.work.saturating_add(1),
            growth.bytes.saturating_add(feature.len()),
        )?;
        self.deferred.borrow_mut().push(Deferred { site, feature });
        Ok(())
    }
    /// Admit each deferred use from `start` on only through its own expansion
    /// frame's allowance list, then drop it.
    fn admit_deferred(&self, report: &Report, start: usize) -> Result<(), Error> {
        let deferred = self.deferred.borrow_mut().split_off(start);
        if deferred.is_empty() {
            return Ok(());
        }
        let mut session = report.macro_allowances();
        for entry in deferred {
            self.charge(1, 0)?;
            let span = report
                .package
                .unit(entry.site.file)
                .configured
                .ast
                .node(entry.site.node)
                .span;
            let feature = crate::resolve::bindings::FeatureName::new(&entry.feature)
                .map_err(Error::unlocated)?;
            let site = crate::expand::stream::Location {
                file: entry.site.file,
                span,
            };
            if !session.allows(site, &feature)? {
                return Err(Error::at(
                    site.file,
                    span.start,
                    format!(
                        "use of unstable library API requires feature `{}`",
                        entry.feature
                    ),
                ));
            }
        }
        Ok(())
    }
    pub(in crate::resolve) fn declaration(
        &self,
        package: &Package,
        site: NodeKey,
        definition: DefId,
        constant: bool,
    ) -> Result<(), Error> {
        self.charge(1, 0)?;
        let Some(mapped) = self.declarations.get(&definition) else {
            return Ok(());
        };
        let mapped = mapped.as_ref().ok_or_else(|| {
            Error::unlocated("imported API availability is unknown in its metadata profile")
        })?;
        self.catalog
            .validate_declaration(mapped.handle, definition, mapped.kind)
            .map_err(Error::unlocated)?;
        let (identity, declaration, edition) = self
            .catalog
            .declaration_coordinate(mapped.handle)
            .map_err(Error::unlocated)?;
        let artifact = &self.inventory.artifacts()[mapped.artifact];
        let descriptor = artifact
            .source_descriptor()
            .ok_or_else(|| Error::unlocated("availability source descriptor disappeared"))?;
        if artifact.identity() != identity
            || declaration != mapped.reference
            || descriptor.edition() != edition
        {
            return Err(Error::unlocated(
                "availability declaration changed its canonical origin",
            ));
        }
        let row = self
            .rows(mapped.artifact)?
            .declarations
            .get(mapped.reference.0 as usize)
            .ok_or_else(|| Error::unlocated("availability declaration coverage is missing"))?;
        self.policy(
            package,
            site,
            mapped.artifact,
            if constant { row.constant } else { row.regular },
        )
    }
    pub(in crate::resolve) fn trace(&self, length: usize) -> Result<Observations, Error> {
        let growth = retained::Growth::vector::<Selection>(0, 0, length);
        self.charge(length.saturating_add(growth.work), growth.bytes)?;
        let mut selected = Vec::new();
        selected
            .try_reserve_exact(length)
            .map_err(|_| Error::unlocated("availability path trace allocation failed"))?;
        if selected.capacity().saturating_mul(size_of::<Selection>()) > growth.bytes {
            return Err(Error::unlocated(
                "availability path trace exceeds prepaid storage",
            ));
        }
        Ok(Observations {
            selected: Some(selected),
            selections_only: true,
            ..Observations::default()
        })
    }
    fn features(&mut self, policies: &[s::AvailabilityPolicy]) -> Result<(), Error> {
        for policy in policies {
            self.feature(&policy.feature)?;
        }
        Ok(())
    }
    /// An imported macro row's `#[unstable]` feature is a library feature its
    /// consumer may request, like an availability row's.
    fn macro_features(
        &mut self,
        macros: &crate::support::crates::exported_macros::Macros,
    ) -> Result<(), Error> {
        let rows = macros.rows().iter().filter_map(|row| row.unstable());
        let builtins = macros.builtins().iter().filter_map(|row| row.unstable());
        let scoped = macros
            .scoped()
            .iter()
            .filter_map(|row| row.definition().unstable());
        for feature in rows.chain(builtins).chain(scoped) {
            self.feature(feature)?;
        }
        Ok(())
    }
    /// The canonical spelling of one written feature name.
    fn spelling<'a>(&self, written: &'a str) -> Result<std::borrow::Cow<'a, str>, Error> {
        // Only normalization allocates, and it pays its own growth. A caller
        // that retains the spelling pays for its bytes.
        self.charge(written.len().saturating_add(1), 0)?;
        let spelling = if written.is_ascii() {
            std::borrow::Cow::Borrowed(written)
        } else {
            std::borrow::Cow::Owned(
                Names::normalize_paid(written, &mut |growth| {
                    self.charge(growth.work, growth.bytes)
                        .map_err(|_| "availability normalization budget exhausted")
                })
                .map_err(Error::unlocated)?,
            )
        };
        crate::resolve::bindings::FeatureName::new(&spelling).map_err(Error::unlocated)?;
        Ok(spelling)
    }
    fn feature(&mut self, written: &str) -> Result<(), Error> {
        let spelling = self.spelling(written)?;
        let feature =
            crate::resolve::bindings::FeatureName::new(&spelling).map_err(Error::unlocated)?;
        if self.features.contains(feature.spelling()) {
            return Ok(());
        }
        let growth =
            retained::Growth::map::<String, ()>(self.features.len(), self.features.capacity(), 1);
        self.charge(
            growth.work.saturating_add(feature.spelling().len()),
            growth.bytes.saturating_add(feature.spelling().len()),
        )?;
        self.features.insert(feature.spelling().to_owned());
        Ok(())
    }
    /// Record every declaration row of one source artifact by its canonical identity.
    fn declarations(
        &mut self,
        imported: &Imported,
        facts: &Resolution,
        artifact: usize,
        identity: &crate::support::crates::CrateIdentity,
        descriptor: s::Descriptor<'_>,
    ) -> Result<(), Error> {
        for (ordinal, _) in descriptor.rows().declarations.iter().enumerate() {
            let reference = DeclarationRef(
                u32::try_from(ordinal)
                    .map_err(|_| Error::unlocated("availability declaration index overflow"))?,
            );
            let definition = imported
                .generic_definition(identity, reference)
                .map_err(Error::unlocated)?;
            let kind = facts
                .definitions
                .get(definition.0 as usize)
                .ok_or_else(|| Error::unlocated("availability canonical declaration is absent"))?
                .kind;
            let handle = self
                .catalog
                .declaration(definition)
                .map_err(Error::unlocated)?;
            self.catalog
                .validate_declaration(handle, definition, kind)
                .map_err(Error::unlocated)?;
            let growth = retained::Growth::map::<DefId, Option<Declaration>>(
                self.declarations.len(),
                self.declarations.capacity(),
                1,
            );
            self.charge(growth.work.saturating_add(1), growth.bytes)?;
            self.declarations.insert(
                definition,
                Some(Declaration {
                    artifact,
                    reference,
                    handle,
                    kind,
                }),
            );
        }
        Ok(())
    }
    fn unsupported(&mut self, definition: DefId) -> Result<(), Error> {
        self.charge(1, 0)?;
        if !self.declarations.contains_key(&definition) {
            let growth = retained::Growth::map::<DefId, Option<Declaration>>(
                self.declarations.len(),
                self.declarations.capacity(),
                1,
            );
            self.charge(growth.work, growth.bytes)?;
            self.declarations.insert(definition, None);
        }
        Ok(())
    }
}

pub(in crate::resolve) fn prepare(
    imported: &Imported,
    facts: &Resolution,
    names: &Names,
) -> Result<Option<Box<Imports>>, Error> {
    let (steps, bytes) = imported.original_reservation_budget();
    let mut budget = Budget { steps, bytes };
    budget.charge(imported.dependencies().artifacts().len(), 0)?;
    if !imported.dependencies().artifacts().iter().any(|artifact| {
        artifact
            .source_descriptor()
            .is_some_and(|descriptor| descriptor.profile().availability())
    }) {
        return Ok(None);
    }
    let catalog = imported
        .origin_catalog()
        .map_err(Error::unlocated)?
        .ok_or_else(|| {
            Error::unlocated("availability import requires its installed origin catalog")
        })?;
    let inventory = imported.shared_dependencies();
    let mut result = Imports {
        catalog,
        inventory: Arc::clone(&inventory),
        declarations: HashMap::default(),
        bindings: HashMap::default(),
        features: HashSet::default(),
        entries: Vec::new(),
        paths: Vec::new(),
        paths_complete: false,
        deferred: RefCell::default(),
        budget: Cell::new(budget),
    };
    result.charge(0, size_of::<Imports>())?;
    for (index, artifact) in inventory.artifacts().iter().enumerate() {
        if let Some(macros) = artifact.macros() {
            result.macro_features(macros)?;
        }
        let Some(descriptor) = artifact.source_descriptor() else {
            continue;
        };
        result.declarations(imported, facts, index, artifact.identity(), descriptor)?;
        if !descriptor.profile().availability() {
            continue;
        }
        result.features(
            &descriptor
                .availability()
                .map_err(Error::unlocated)?
                .policies,
        )?;
        for (ordinal, row) in descriptor
            .module_contracts()
            .map_err(Error::unlocated)?
            .rows()
            .namespace
            .bindings
            .iter()
            .enumerate()
        {
            install_binding(&mut result, imported, facts, names, index, ordinal, row)?;
        }
    }
    for definition in imported.generic_definitions() {
        result.unsupported(definition.map_err(Error::unlocated)?)?;
    }
    for (_, definition) in imported.definitions() {
        result.unsupported(definition)?;
    }
    Ok(Some(Box::new(result)))
}
fn install_binding(
    result: &mut Imports,
    imported: &Imported,
    facts: &Resolution,
    names: &Names,
    artifact: usize,
    index: usize,
    row: &s::ModuleBindingRow,
) -> Result<(), Error> {
    let owner = result.inventory.artifacts()[artifact].identity();
    let module = imported
        .generic_definition(owner, row.module)
        .map_err(Error::unlocated)?;
    let artifact_row = &result.inventory.artifacts()[artifact];
    let target_owner = if row.target.artifact.0 == 0 {
        owner
    } else {
        artifact_row
            .dependencies()
            .get((row.target.artifact.0 - 1) as usize)
            .ok_or_else(|| Error::unlocated("availability binding target artifact is absent"))?
    };
    let target = imported
        .generic_definition(target_owner, row.target.declaration)
        .map_err(Error::unlocated)?;
    let scope = facts.definitions[module.0 as usize]
        .members
        .ok_or_else(|| Error::unlocated("availability module has no canonical member scope"))?;
    let name = names
        .lookup_paid(&row.name, &mut |growth| {
            result
                .charge(growth.work, growth.bytes)
                .map_err(|_| "availability name budget exhausted")
        })
        .map_err(Error::unlocated)?
        .ok_or_else(|| Error::unlocated("availability binding name was not installed"))?;
    let namespace = match row.namespace {
        crate::support::crates::Namespace::Type => Namespace::Type,
        crate::support::crates::Namespace::Value => Namespace::Value,
    };
    let key = (scope, name, namespace, ItemContext::ROOT);
    if facts
        .bindings
        .get_key(key)
        .is_none_or(|binding| binding.target != target)
    {
        return Err(Error::unlocated(
            "availability binding differs from installed namespace",
        ));
    }
    let growth = retained::Growth::map::<BindingKey, Binding>(
        result.bindings.len(),
        result.bindings.capacity(),
        1,
    );
    result.charge(growth.work.saturating_add(1), growth.bytes)?;
    if result
        .bindings
        .insert(
            key,
            Binding {
                artifact,
                index,
                target,
                source: row.source,
            },
        )
        .is_some()
    {
        return Err(Error::unlocated(
            "availability canonical namespace has duplicate policy keys",
        ));
    }
    Ok(())
}

pub(crate) fn check(
    report: &crate::resolve::expansion::Report,
    site: NodeKey,
    definition: DefId,
    constant: bool,
) -> Result<(), Error> {
    let Some(availability) = owner(&report.package, report.resolution.crates.as_deref())? else {
        return Ok(());
    };
    let start = availability.deferred.borrow().len();
    availability.declaration(&report.package, site, definition, constant)?;
    availability.admit_deferred(report, start)
}
/// After ordinary resolution, admit every use deferred inside an expansion
/// through its frame's allowance list.
/// # Errors
/// Rejects a deferred use that no enclosing frame admits, by its feature.
pub(in crate::resolve) fn admit_expanded(report: &Report) -> Result<(), Error> {
    let Some(availability) = owner(&report.package, report.resolution.crates.as_deref())? else {
        return Ok(());
    };
    availability.admit_deferred(report, 0)
}
pub(in crate::resolve) fn check_all(
    package: &Package,
    facts: &mut Resolution,
) -> Result<(), Error> {
    if let Some(imports) = facts
        .crates
        .as_deref_mut()
        .and_then(|i| i.availability.as_deref_mut())
    {
        imports.complete_paths()?;
    }
    let Some(availability) = owner(package, facts.crates.as_deref())? else {
        return Ok(());
    };
    availability.check_uses(package, facts)
}
fn owner<'a>(
    package: &Package,
    imported: Option<&'a Imported>,
) -> Result<Option<&'a Imports>, Error> {
    let availability = imported.and_then(|imports| imports.availability.as_deref());
    if let Some(availability) = availability {
        availability.charge(1, 0)?;
    }
    let required = package
        .source_client()
        .map(|client| client.availability_required(imported.map(Imported::dependencies)))
        .transpose()
        .map_err(Error::unlocated)?
        .unwrap_or(false);
    if required && availability.is_none() {
        return Err(Error::unlocated(
            "source API availability requires its actual imported owner",
        ));
    }
    if let (Some(availability), Some(imported)) = (availability, imported) {
        availability.same_owner(imported)?;
    }
    Ok(availability)
}

/// Only the explicit v6 source contract excludes deprecated declaration rows.
/// Missing/legacy origin profiles retain unknown knowledge, even through aliases.
pub(crate) fn declaration_deprecation_absent(
    report: &crate::resolve::expansion::Report,
    definition: DefId,
) -> Result<bool, Error> {
    let Some(imports) = owner(&report.package, report.resolution.crates.as_deref())? else {
        return Ok(false);
    };
    imports.declaration_deprecation_absent(definition)
}
