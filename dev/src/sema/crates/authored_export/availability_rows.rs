//! Actual completed policy owners join the emitted declaration and binding census.
use super::{Declarations, DefId, Report, Stage, g, s};
use crate::{
    resolve::{
        model::{DefKind, Namespace, Origin},
        stability::{Inventory, Level, Policy, PolicyKind, Since},
        unit::NodeKey,
    },
    support::{crates::owner_catalog::DeclarationRef, hash::HashMap, retained},
    syntax::ast::Kind,
};

#[derive(Clone, Copy)]
struct SourceOwner {
    owner: DeclarationRef,
    interval: [u32; 2],
}
struct Origins {
    sources: HashMap<NodeKey, Vec<SourceOwner>>,
    ancestry: s::AvailabilityAncestry,
    definitions: Vec<Option<DefId>>,
}
#[derive(Clone, Copy)]
enum Subject {
    Declaration(DeclarationRef),
    Binding {
        owner: DeclarationRef,
        index: u32,
        site: u32,
    },
}
impl Subject {
    const fn owner(self) -> DeclarationRef {
        match self {
            Self::Declaration(owner) | Self::Binding { owner, .. } => owner,
        }
    }
}
#[derive(Default)]
struct Policies {
    rows: Vec<s::AvailabilityPolicy>,
    index: HashMap<(usize, s::AvailabilityOrigin), s::AvailabilityRef>,
}
impl Stage {
    pub(crate) fn availability_rows(
        &mut self,
        report: &Report,
        facts: &Declarations,
        modules: &s::ModuleContractRows,
    ) -> Result<s::AvailabilityRows, &'static str> {
        let inventory = report
            .package
            .library_policy()
            .and_then(crate::resolve::stability::Library::staged_inventory)
            .ok_or("availability collection requires its completed source inventory")?;
        self.availability_inventory(report, facts, modules, inventory)
    }

    /// Version nine also exports authenticated ordinary stability absence.
    pub(crate) fn conversion_availability_rows(
        &mut self,
        report: &Report,
        facts: &Declarations,
        modules: &s::ModuleContractRows,
    ) -> Result<s::AvailabilityRows, &'static str> {
        let Some(library) = report.package.library_policy() else {
            return self.unstaged_availability(report, facts, modules);
        };
        let inventory = library
            .inventory()
            .ok_or("conversion availability lacks its completed source inventory")?;
        self.availability_inventory(report, facts, modules, inventory)
    }

    /// Successful reservation capture has checked every active stability
    /// attribute. With no Library request, each such attribute is a hard error;
    /// absence follows from that completed current producer, never an import.
    fn unstaged_availability(
        &mut self,
        report: &Report,
        facts: &Declarations,
        modules: &s::ModuleContractRows,
    ) -> Result<s::AvailabilityRows, &'static str> {
        self.require_attribute_context(report, facts)
            .map_err(|_| "unstaged availability lost its completed source capture")?;
        let count = self.rows.declarations.len();
        let bindings = modules.namespace.bindings.len();
        self.charge(count.saturating_add(bindings))?;
        self.reserve(
            count
                .saturating_mul(size_of::<s::DeclarationAvailability>())
                .saturating_add(bindings.saturating_mul(size_of::<s::BindingAvailability>())),
        )?;
        let mut rows = s::AvailabilityRows {
            policies: Vec::new(),
            declarations: Vec::with_capacity(count),
            bindings: Vec::with_capacity(bindings),
        };
        for index in 0..count {
            rows.declarations.push(s::DeclarationAvailability {
                declaration: DeclarationRef(
                    u32::try_from(index).map_err(|_| "unstaged declaration identity overflow")?,
                ),
                regular: None,
                constant: None,
            });
        }
        for (index, binding) in modules.namespace.bindings.iter().enumerate() {
            if binding.target.artifact != self.rows.policy.provider {
                return Err("foreign binding requires its actual availability producer");
            }
            rows.bindings.push(s::BindingAvailability {
                binding: u32::try_from(index).map_err(|_| "unstaged binding identity overflow")?,
                regular: None,
            });
        }
        Ok(rows)
    }

    fn availability_inventory(
        &mut self,
        report: &Report,
        facts: &Declarations,
        modules: &s::ModuleContractRows,
        inventory: &Inventory,
    ) -> Result<s::AvailabilityRows, &'static str> {
        self.require_attribute_context(report, facts)
            .map_err(|_| "availability collection lost its actual source owner")?;
        if inventory.has_implications() {
            return Err(
                "source implied feature relationships require complete wider policy transport",
            );
        }
        let census = self
            .rows
            .declarations
            .len()
            .saturating_mul(2)
            .saturating_add(modules.namespace.bindings.len());
        self.charge(census)?;
        self.reserve(
            self.rows
                .declarations
                .len()
                .saturating_mul(4 * size_of::<s::DeclarationAvailability>())
                .saturating_add(
                    modules
                        .namespace
                        .bindings
                        .len()
                        .saturating_mul(4 * size_of::<s::BindingAvailability>()),
                ),
        )?;
        let origins = self.availability_origins(report, modules, inventory.policies().len())?;
        let mut policies = Policies::default();
        let mut declarations = vec![None; self.rows.declarations.len()];
        for index in 0..self.definitions.len() {
            self.charge(1)?;
            let Some(declaration) = self.definitions[index] else {
                continue;
            };
            let actual =
                DefId(u32::try_from(index).map_err(|_| "availability declaration overflow")?);
            declarations[declaration.0 as usize] = Some(self.availability_declaration(
                report,
                inventory,
                &origins,
                &mut policies,
                actual,
                declaration,
            )?);
        }
        let mut bindings = Vec::with_capacity(modules.namespace.bindings.len());
        for (index, binding) in modules.namespace.bindings.iter().enumerate() {
            self.charge(1)?;
            let regular = if binding.source == s::BindingSource::Declaration {
                if binding.target.artifact != self.rows.policy.provider {
                    return Err("direct availability binding is foreign");
                }
                declarations
                    .get(binding.target.declaration.0 as usize)
                    .copied()
                    .flatten()
                    .ok_or("availability binding has no completed target declaration")?
                    .regular
            } else {
                let import = actual_import(report, self, &origins, binding)?;
                self.availability_policy(
                    report,
                    inventory,
                    &origins,
                    &mut policies,
                    Subject::Binding {
                        owner: binding.module,
                        index: u32::try_from(index).map_err(|_| "availability binding overflow")?,
                        site: binding.source_site,
                    },
                    inventory.import_id(import)?,
                )?
            };
            bindings.push(s::BindingAvailability {
                binding: u32::try_from(index).map_err(|_| "availability binding overflow")?,
                regular,
            });
        }
        let declarations = declarations
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or("availability declaration census is incomplete")?;
        Ok(s::AvailabilityRows {
            policies: policies.rows,
            declarations,
            bindings,
        })
    }
    fn availability_declaration(
        &mut self,
        report: &Report,
        inventory: &Inventory,
        origins: &Origins,
        policies: &mut Policies,
        actual: DefId,
        declaration: DeclarationRef,
    ) -> Result<s::DeclarationAvailability, &'static str> {
        let (regular, constant) = inventory.declaration_ids(actual)?;
        let regular = self.availability_policy(
            report,
            inventory,
            origins,
            policies,
            Subject::Declaration(declaration),
            regular,
        )?;
        let constant = self.availability_policy(
            report,
            inventory,
            origins,
            policies,
            Subject::Declaration(declaration),
            constant,
        )?;
        Ok(s::DeclarationAvailability {
            declaration,
            regular,
            constant,
        })
    }
    fn availability_origins(
        &mut self,
        report: &Report,
        modules: &s::ModuleContractRows,
        policies: usize,
    ) -> Result<Origins, &'static str> {
        let count = self
            .rows
            .declarations
            .len()
            .saturating_mul(2)
            .saturating_add(modules.namespace.bindings.len())
            .saturating_add(policies);
        self.charge(count.saturating_mul(4))?;
        self.scratch(
            count.saturating_mul(
                8 * (size_of::<SourceOwner>()
                    + size_of::<NodeKey>()
                    + size_of::<Vec<SourceOwner>>()),
            ) + size_of::<Origins>(),
        )?;
        let mut work = g::Work {
            steps: self.steps,
            ..g::Work::default()
        };
        let ancestry = s::AvailabilityAncestry::new(
            &self.rows.declarations,
            modules.namespace.root,
            self.bytes.saturating_add(self.scratch),
            &mut work,
        );
        self.charge(work.steps.saturating_sub(self.steps))?;
        let mut result = Origins {
            sources: HashMap::default(),
            ancestry: ancestry?,
            definitions: vec![None; self.rows.declarations.len()],
        };
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            self.availability_source(report, &mut result, index, definition)?;
        }
        for origins in result.sources.values_mut() {
            self.charge(
                origins
                    .len()
                    .saturating_mul(origins.len().max(1).ilog2() as usize + 2),
            )?;
            origins.sort_unstable_by_key(|origin| origin.interval[0]);
            origins.dedup_by_key(|origin| origin.owner);
            if origins
                .windows(2)
                .any(|pair| pair[0].interval[1] > pair[1].interval[0])
            {
                return Err("availability source incarnation has ambiguous nested owners");
            }
        }
        Ok(result)
    }
    fn availability_source(
        &mut self,
        report: &Report,
        result: &mut Origins,
        index: usize,
        definition: &crate::resolve::model::Definition,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let Some(owner) = self.definitions[index] else {
            return Ok(());
        };
        result.definitions[owner.0 as usize] = Some(DefId(
            u32::try_from(index).map_err(|_| "availability definition overflow")?,
        ));
        if let Some(source) = definition.node {
            let actual = matches!(
                definition.kind,
                DefKind::Module | DefKind::Trait | DefKind::Function
            ) || (definition.kind == DefKind::SelfType
                && matches!(report.package.unit(source.file).configured.ast.node(source.node).kind,Kind::Item(item) if matches!(report.package.unit(source.file).configured.ast.items[item as usize].kind,crate::syntax::item::ItemKind::Impl {..})));
            if actual {
                self.source_origin(report, result, source, owner)?;
            }
        }
        if definition.kind == DefKind::Module {
            let actual = DefId(u32::try_from(index).map_err(|_| "availability module overflow")?);
            let module = crate::resolve::impl_restrictions::module(
                &report.package,
                &report.resolution,
                actual,
            )
            .map_err(|_| "availability module has no actual source owner")?;
            let module = &report.package.modules[module.0 as usize];
            self.source_origin(
                report,
                result,
                NodeKey {
                    file: module.file,
                    node: module.root,
                },
                owner,
            )?;
        }
        Ok(())
    }
    fn source_origin(
        &mut self,
        report: &Report,
        result: &mut Origins,
        mut source: NodeKey,
        owner: DeclarationRef,
    ) -> Result<(), &'static str> {
        while let Kind::Attributed { value, .. } = report
            .package
            .unit(source.file)
            .configured
            .ast
            .node(source.node)
            .kind
        {
            self.charge(1)?;
            source.node = value;
        }
        let interval = result
            .ancestry
            .interval(owner)
            .ok_or("availability owner has no actual ancestry interval")?;
        result
            .sources
            .entry(source)
            .or_default()
            .push(SourceOwner { owner, interval });
        Ok(())
    }
    fn availability_policy(
        &mut self,
        report: &Report,
        inventory: &Inventory,
        origins: &Origins,
        policies: &mut Policies,
        subject: Subject,
        reference: Option<usize>,
    ) -> Result<Option<s::AvailabilityRef>, &'static str> {
        let Some(reference) = reference else {
            return Ok(None);
        };
        self.charge(1)?;
        let policy = inventory
            .policies()
            .get(reference)
            .ok_or("availability source policy disappeared")?;
        let origin = self.policy_origin(report, origins, subject, policy)?;
        if let Some(found) = policies.index.get(&(reference, origin)) {
            return Ok(Some(*found));
        }
        let mut growth = retained::Growth::vector::<s::AvailabilityPolicy>(
            policies.rows.len(),
            policies.rows.capacity(),
            1,
        );
        growth.include(retained::Growth::map::<
            (usize, s::AvailabilityOrigin),
            s::AvailabilityRef,
        >(policies.index.len(), policies.index.capacity(), 1));
        self.charge(growth.work.saturating_add(policy.feature.len()))?;
        self.reserve(growth.bytes.saturating_add(policy.feature.len()))?;
        let level = match &policy.level {
            Level::Stable(since) => s::AvailabilityLevel::Stable(since_value(*since)),
            Level::StableWithCompatibility { since, note } => {
                self.charge(note.message.len())?;
                self.reserve(note.message.len())?;
                s::AvailabilityLevel::StableWithCompatibility {
                    since: since_value(*since),
                    message: note.message.to_string(),
                }
            }
            Level::Unstable { issue, reason } => {
                self.charge(reason.as_deref().map_or(0, str::len))?;
                self.reserve(reason.as_deref().map_or(0, str::len))?;
                s::AvailabilityLevel::Unstable {
                    issue: *issue,
                    reason: reason.as_deref().map(str::to_owned),
                }
            }
        };
        let id = s::AvailabilityRef(
            u32::try_from(policies.rows.len())
                .map_err(|_| "availability policy inventory overflow")?,
        );
        policies.rows.push(s::AvailabilityPolicy {
            kind: match policy.kind {
                PolicyKind::Regular => s::AvailabilityKind::Regular,
                PolicyKind::Const => s::AvailabilityKind::Const,
            },
            feature: policy.feature.to_string(),
            level,
            origin,
        });
        policies.index.insert((reference, origin), id);
        Ok(Some(id))
    }
    fn policy_origin(
        &mut self,
        report: &Report,
        origins: &Origins,
        subject: Subject,
        policy: &Policy,
    ) -> Result<s::AvailabilityOrigin, &'static str> {
        let owner = subject.owner();
        if let Some(candidates) = origins.sources.get(&policy.origin) {
            let interval = origins
                .ancestry
                .interval(owner)
                .ok_or("availability subject has no actual owner")?;
            let mut start = 0;
            let mut end = candidates.len();
            while start < end {
                self.charge(1)?;
                let middle = start + (end - start) / 2;
                if candidates[middle].interval[0] <= interval[0] {
                    start = middle + 1;
                } else {
                    end = middle;
                }
            }
            if let Some(origin) = start
                .checked_sub(1)
                .and_then(|index| candidates.get(index))
                .filter(|origin| interval[1] <= origin.interval[1])
            {
                let site = self.site(origin.owner, policy.origin)?;
                return Ok(s::AvailabilityOrigin::Declaration {
                    owner: origin.owner,
                    site,
                });
            }
        }
        if let Subject::Binding { index, site, .. } = subject
            && policy.kind == PolicyKind::Regular
            && let Kind::Item(item) = report
                .package
                .unit(policy.origin.file)
                .configured
                .ast
                .node(policy.origin.node)
                .kind
            && matches!(
                report.package.unit(policy.origin.file).configured.ast.items[item as usize].kind,
                crate::syntax::item::ItemKind::Use(_)
            )
        {
            return Ok(s::AvailabilityOrigin::Binding {
                binding: index,
                site,
            });
        }
        Err("availability policy has no actual emitted declaration/import origin")
    }
}
fn actual_import(
    report: &Report,
    stage: &mut Stage,
    origins: &Origins,
    row: &s::ModuleBindingRow,
) -> Result<usize, &'static str> {
    // The emitted binding retains actual scope/name/namespace; no target spelling is guessed.
    stage.charge(1)?;
    let actual = origins
        .definitions
        .get(row.module.0 as usize)
        .copied()
        .flatten()
        .ok_or("availability module declaration is absent")?;
    let scope = report.resolution.definitions[actual.0 as usize]
        .members
        .ok_or("availability module has no actual scope")?;
    let name = report
        .package
        .names
        .lookup_paid(&row.name, &mut |growth| {
            stage.charge(growth.work)?;
            stage.reserve(growth.bytes)
        })?
        .ok_or("availability binding name disappeared")?;
    let namespace = match row.namespace {
        crate::support::crates::Namespace::Type => Namespace::Type,
        crate::support::crates::Namespace::Value => Namespace::Value,
    };
    let binding = report
        .resolution
        .bindings
        .get(scope, name, namespace)
        .ok_or("availability binding has no actual resolved source")?;
    if stage.address(binding.target)? != row.target {
        return Err("availability import changed its actual resolved target");
    }
    let index = match (binding.origin, row.source) {
        (Origin::Import(index), s::BindingSource::Import)
        | (Origin::Glob(index), s::BindingSource::Glob) => index as usize,
        _ => return Err("availability import changed its actual binding kind"),
    };
    let import = report
        .resolution
        .imports
        .get(index)
        .filter(|import| {
            import.scope == scope && import.glob == (row.source == s::BindingSource::Glob)
        })
        .ok_or("availability import changed its actual source owner")?;
    if stage.known_site(row.module, import.node)? != Some(row.source_site) {
        return Err("availability import changed its actual leaf site");
    }
    Ok(index)
}

fn since_value(since: Since) -> s::AvailabilitySince {
    match since {
        Since::Current => s::AvailabilitySince::Current,
        Since::Version {
            major,
            minor,
            patch,
        } => s::AvailabilitySince::Version {
            major,
            minor,
            patch,
        },
    }
}
