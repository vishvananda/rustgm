//! Actual resolved bindings and clauses produce complete version-two coordinates once.
#[cfg(test)]
#[path = "../../../../tests/invariants/reexported_variants.rs"]
mod variant_tests;
use super::{Declarations, DefId, DefKind, Namespace, Report, Stage, g, s};
use crate::{
    resolve::{
        impl_restrictions,
        model::{Access, Binding, Origin, Target},
        stability::{Inventory, Library},
        unit::NodeKey,
    },
    support::{
        crates::{Namespace as WireNamespace, owner_catalog::DeclarationRef},
        retained::Growth,
    },
    syntax::ast::Kind,
};
use std::num::NonZeroU32;

/// The namespace emitter's resolved policy indices follow the same permutation
/// as their rows and live only until availability has been collected.
pub(crate) struct AuthoredModules {
    rows: s::ModuleContractRows,
    policies: Vec<Option<NonZeroU32>>,
    policy_bytes: usize,
}
impl AuthoredModules {
    pub(crate) const fn rows(&self) -> &s::ModuleContractRows {
        &self.rows
    }
    pub(super) fn import_policy(&self, index: usize) -> Result<Option<usize>, &'static str> {
        self.policies
            .get(index)
            .map(|policy| policy.map(|id| id.get() as usize - 1))
            .ok_or("availability binding has no resolved policy entry")
    }
    pub(crate) fn into_rows(
        self,
        stage: &mut Stage,
    ) -> Result<s::ModuleContractRows, &'static str> {
        drop(self.policies);
        stage.release_temporary(self.policy_bytes)?;
        Ok(self.rows)
    }
    fn record_policy(
        &mut self,
        stage: &mut Stage,
        inventory: &Inventory,
        binding: &Binding,
        source: s::BindingSource,
    ) -> Result<(), &'static str> {
        stage.charge(1)?;
        let regular = match binding.origin {
            Origin::Direct if source == s::BindingSource::Import => {
                let (regular, constant) = inventory.declaration_ids(binding.target)?;
                if constant.is_some() {
                    return Err("availability self alias changed its actual policy");
                }
                regular
            }
            Origin::Import(index) | Origin::Glob(index) => inventory.import_id(index as usize)?,
            Origin::Direct => None,
        };
        let regular = regular.map(policy_coordinate).transpose()?;
        if self.policies.len() == self.policies.capacity() {
            let growth = Growth::vector::<Option<NonZeroU32>>(
                self.policies.len(),
                self.policies.capacity(),
                1,
            );
            stage.charge(growth.work)?;
            stage.reserve(growth.bytes)?;
            self.policy_bytes = self.policy_bytes.saturating_add(growth.bytes);
        }
        self.policies.push(regular);
        Ok(())
    }
}

fn policy_coordinate(index: usize) -> Result<NonZeroU32, &'static str> {
    u32::try_from(index.saturating_add(1))
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or("availability import policy ordinal overflow")
}

struct Modules {
    rows: Vec<Option<DeclarationRef>>,
    declarations: Vec<Option<DefId>>,
}
impl Modules {
    fn local(
        &self,
        module: crate::resolve::package::ModuleId,
    ) -> Result<DeclarationRef, &'static str> {
        self.rows
            .get(module.0 as usize)
            .copied()
            .flatten()
            .ok_or("module policy has no actual source coordinate")
    }
}
impl Stage {
    pub(crate) fn module_contract_rows(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<AuthoredModules, &'static str> {
        self.reserve(
            report
                .package
                .modules
                .len()
                .saturating_mul(size_of::<Option<DeclarationRef>>())
                .saturating_add(
                    self.rows
                        .declarations
                        .len()
                        .saturating_mul(size_of::<Option<DefId>>()),
                ),
        )?;
        self.charge(
            report
                .package
                .modules
                .len()
                .saturating_add(self.rows.declarations.len())
                .saturating_add(self.definitions.len()),
        )?;
        let mut index = Modules {
            rows: vec![None; report.package.modules.len()],
            declarations: vec![None; self.rows.declarations.len()],
        };
        let root = self.declaration(
            report
                .resolution
                .crate_root
                .ok_or("module policy has no actual source root")?,
        )?;
        let mut result = AuthoredModules {
            rows: s::ModuleContractRows {
                namespace: s::ModuleNamespace {
                    root,
                    modules: Vec::new(),
                    bindings: Vec::new(),
                },
                domains: Vec::new(),
            },
            policies: Vec::new(),
            policy_bytes: 0,
        };
        for (actual, row) in self.definitions.iter().enumerate() {
            if let Some(row) = row {
                index.declarations[row.0 as usize] = Some(DefId(
                    u32::try_from(actual).map_err(|_| "module owner overflow")?,
                ));
            }
        }

        for (actual, definition) in report.resolution.definitions.iter().enumerate() {
            self.charge(1)?;
            if definition.kind != DefKind::Module {
                continue;
            }
            let Some(row) = self.definitions[actual] else {
                continue;
            };
            let actual = DefId(u32::try_from(actual).map_err(|_| "module declaration overflow")?);
            let module = impl_restrictions::module(&report.package, &report.resolution, actual)?;
            if index.rows[module.0 as usize].replace(row).is_some() {
                return Err("module policy has duplicate actual modules");
            }
            let site = definition.node.map_or(Ok(0), |node| self.site(row, node))?;
            self.reserve(4 * size_of::<s::ModuleRow>())?;
            result.rows.namespace.modules.push(s::ModuleRow {
                declaration: row,
                declaration_site: site,
            });
        }
        self.module_bindings(report, facts, &index, &mut result)?;
        self.trait_domains(report, facts, &index, &mut result.rows.domains)?;
        self.scratch(
            result
                .rows
                .namespace
                .bindings
                .len()
                .saturating_mul(2 * size_of::<usize>())
                .saturating_add(2 * size_of::<Vec<usize>>()),
        )?;
        let mut work = g::Work {
            steps: self.work().steps,
            ..g::Work::default()
        };
        let ordered = result.rows.canonicalize(&mut work, |left, right| {
            if !result.policies.is_empty() {
                result.policies.swap(left, right);
            }
        });
        let delta = work.steps.saturating_sub(self.work().steps);
        self.charge(delta)?;
        ordered?;
        Ok(result)
    }
    fn module_bindings(
        &mut self,
        report: &Report,
        facts: &Declarations,
        index: &Modules,
        modules: &mut AuthoredModules,
    ) -> Result<(), &'static str> {
        let inventory = report
            .package
            .library_policy()
            .and_then(Library::inventory)
            .filter(|_| self.mode.availability());
        for position in 0..modules.rows.namespace.modules.len() {
            let module = modules.rows.namespace.modules[position];
            self.charge(1)?;
            let actual = index.declarations[module.declaration.0 as usize]
                .ok_or("module declaration origin is absent")?;
            let scope = report.resolution.definitions[actual.0 as usize]
                .members
                .ok_or("actual module has no member scope")?;
            for (name, namespace, binding) in report.resolution.bindings.scope(scope) {
                self.charge(1)?;
                if binding.ambiguous {
                    return Err("module contract binding is ambiguous");
                }
                if self.erased_alias_binding(report, facts, scope, namespace, binding)?
                    || self.variant_alias_binding(report, scope, name, namespace, binding)?
                {
                    continue;
                }
                let namespace = match namespace {
                    Namespace::Type => WireNamespace::Type,
                    Namespace::Value => WireNamespace::Value,
                    // Macro bindings travel only in the macro table, which
                    // rejects every unsupported macro binding by name.
                    Namespace::Macro => continue,
                    _ => return Err("module contract namespace is unsupported"),
                };
                let visibility = match binding.access {
                    Access::Public => s::ModuleVisibility::Public,
                    Access::Within(module) => s::ModuleVisibility::Within(index.local(module)?),
                    Access::ExternalPrivate => {
                        return Err("foreign private module binding is unsupported");
                    }
                };
                let (target, alias) = self.module_binding_target(report, binding.target)?;
                let (source, node) = binding_origin(report, scope, binding, alias)?;
                let source_site = self.site(module.declaration, node)?;
                if let Some(inventory) = inventory {
                    modules.record_policy(self, inventory, binding, source)?;
                }
                let name = report.package.names.get(name);
                self.charge(name.len())?;
                self.reserve(4 * (size_of::<s::ModuleBindingRow>() + name.len()))?;
                modules.rows.namespace.bindings.push(s::ModuleBindingRow {
                    module: module.declaration,
                    name: name.to_owned(),
                    namespace,
                    target,
                    visibility,
                    source,
                    source_site,
                });
            }
        }
        Ok(())
    }
    /// The alias borrows the root's members and exports a binding to that root.
    pub(super) fn private_self_alias(
        &mut self,
        report: &Report,
        actual: DefId,
    ) -> Result<(DefId, NodeKey), &'static str> {
        self.charge(4)?;
        let definition = &report.resolution.definitions[actual.0 as usize];
        let module = crate::resolve::package::ModuleId(0);
        if !self.declaration_policies() || definition.access != Access::Within(module) {
            return Err("source extern crate declaration requires its wider profile");
        }
        let source = crate::resolve::root_self_alias(
            &report.package,
            &report.resolution,
            definition.name,
            actual,
        )
        .ok_or("source extern crate requires its actual private root self alias")?;
        let root = report
            .resolution
            .crate_root
            .filter(|root| {
                let row = &report.resolution.definitions[root.0 as usize];
                row.kind == DefKind::Module && row.members == definition.members
            })
            .ok_or("source self alias lost its actual root declaration")?;
        Ok((root, source))
    }
    fn module_binding_target(
        &mut self,
        report: &Report,
        actual: DefId,
    ) -> Result<(s::DeclarationAddress, Option<NodeKey>), &'static str> {
        if report.resolution.definitions[actual.0 as usize].kind == DefKind::ExternCrate {
            let (root, source) = self.private_self_alias(report, actual)?;
            Ok((self.address(root)?, Some(source)))
        } else {
            Ok((self.address(actual)?, None))
        }
    }
    /// Only the defining private name of a positively checked transparent alias
    /// disappears. Imports/reexports still require their genuine named row.
    fn erased_alias_binding(
        &mut self,
        report: &Report,
        facts: &Declarations,
        scope: crate::resolve::model::ScopeId,
        namespace: Namespace,
        binding: &Binding,
    ) -> Result<bool, &'static str> {
        if !self.conversion_roles() || namespace != Namespace::Type {
            return Ok(false);
        }
        self.charge(2)?;
        let declaration = report
            .resolution
            .definitions
            .get(binding.target.0 as usize)
            .ok_or("module binding lost its actual source declaration")?;
        if declaration.kind != DefKind::TypeAlias {
            return Ok(false);
        }
        let row = self
            .definitions
            .get(binding.target.0 as usize)
            .copied()
            .flatten();
        if row.is_some_and(|row| {
            self.rows.declarations[row.0 as usize].definition.kind == s::DefinitionKind::TypeAlias
        }) {
            return Ok(false);
        }
        if row.is_some()
            || !matches!(declaration.access, Access::Within(_))
            || !matches!(binding.origin, Origin::Direct)
            || declaration.scope != scope
            || binding.access != declaration.access
        {
            return Err("named alias binding transport requires its actual wider profile");
        }
        // The absent row is not proof. Reauthenticate the same source node,
        // complete closed type and nongeneric/nonassociated private owner.
        self.closed_alias(report, facts, binding.target)?;
        Ok(true)
    }
    /// Version thirteen names a unit or tuple variant once, by its
    /// value-namespace constructor declaration. A re-exported tuple or unit
    /// variant keeps that binding; its type-namespace alias is not emitted, as
    /// the enum's member scope omits it, and only beside the same variant's
    /// value binding. A record variant's own type-namespace declaration has a
    /// row, so its re-export is an ordinary type-namespace binding.
    fn variant_alias_binding(
        &mut self,
        report: &Report,
        scope: crate::resolve::model::ScopeId,
        name: crate::resolve::names::NameId,
        namespace: Namespace,
        binding: &Binding,
    ) -> Result<bool, &'static str> {
        let definitions = &report.resolution.definitions;
        let target = definitions
            .get(binding.target.0 as usize)
            .ok_or("module binding lost its actual source declaration")?;
        if target.kind != DefKind::Variant {
            return Ok(false);
        }
        self.charge(2)?;
        if !self.body_nodes() {
            return Err("variant re-exports require the version-13 module-bodies profile");
        }
        if namespace != Namespace::Type
            || self
                .definitions
                .get(binding.target.0 as usize)
                .copied()
                .flatten()
                .is_some()
        {
            return Ok(false);
        }
        let constructor = report
            .resolution
            .bindings
            .get(scope, name, Namespace::Value)
            .filter(|value| value.access == binding.access && !value.ambiguous)
            .and_then(|value| definitions.get(value.target.0 as usize));
        if constructor.is_none_or(|value| {
            value.kind != DefKind::Variant
                || value.namespace != Namespace::Value
                || value.node.is_none()
                || value.node != target.node
        }) {
            return Err("type-namespace variant re-export requires its value constructor binding");
        }
        Ok(true)
    }
    fn trait_domains(
        &mut self,
        report: &Report,
        facts: &Declarations,
        index: &Modules,
        domains: &mut Vec<s::TraitDomainRow>,
    ) -> Result<(), &'static str> {
        for slot in 0..self.rows.traits.len() {
            self.charge(1)?;
            let row = self.rows.traits[slot].declaration;
            let actual = index.declarations[row.0 as usize]
                .ok_or("trait domain has no actual declaration")?;
            let declaration = &report.resolution.definitions[actual.0 as usize];
            let node = declaration
                .node
                .ok_or("trait domain cannot infer imported policy")?;
            let defining =
                index.local(report.resolution.scopes[declaration.scope.0 as usize].module)?;
            let declaration_site = self.site(row, node)?;
            let domain = if let Some(clause) = impl_restrictions::clause(report, actual)
                .map_err(|_| "trait domain clause is invalid")?
            {
                let ast = &report.package.unit(clause.file).configured.ast;
                let Kind::ImplRestriction { path } = ast.node(clause.node).kind else {
                    return Err("trait domain clause changed its actual kind");
                };
                let path = NodeKey {
                    file: clause.file,
                    node: path,
                };
                let Target::Definition(target) = facts
                    .node(path)
                    .and_then(|node| node.reference)
                    .ok_or("trait domain module is unresolved")?
                else {
                    return Err("trait domain target is not a module");
                };
                let target =
                    impl_restrictions::module(&report.package, &report.resolution, target)?;
                self.rows.traits[slot].flags = s::TraitFlags::from_bits(
                    self.rows.traits[slot].flags.bits() | s::TraitFlags::RESTRICTED_IMPL,
                )?;
                s::ImplementationDomain::RestrictedTo {
                    module: index.local(target)?,
                    clause_site: self.site(row, clause)?,
                    path_site: self.site(row, path)?,
                }
            } else {
                s::ImplementationDomain::Unrestricted
            };
            self.reserve(4 * size_of::<s::TraitDomainRow>())?;
            domains.push(s::TraitDomainRow {
                declaration: row,
                declaration_site,
                defining_module: defining,
                domain,
            });
        }
        Ok(())
    }
}
fn binding_origin(
    report: &Report,
    scope: super::ScopeId,
    binding: &Binding,
    alias: Option<NodeKey>,
) -> Result<(s::BindingSource, NodeKey), &'static str> {
    if let (Origin::Direct, Some(site)) = (binding.origin, alias) {
        return Ok((s::BindingSource::Import, site));
    }
    match binding.origin {
        Origin::Direct => Ok((
            s::BindingSource::Declaration,
            report
                .resolution
                .definitions
                .get(binding.target.0 as usize)
                .and_then(|row| row.node)
                .ok_or("module direct binding has no actual source declaration")?,
        )),
        Origin::Import(index) | Origin::Glob(index) => {
            let import = report
                .resolution
                .imports
                .get(index as usize)
                .filter(|import| {
                    import.scope == scope
                        && import.glob == matches!(binding.origin, Origin::Glob(_))
                })
                .ok_or("module alias changed its actual import owner")?;
            Ok((
                if import.glob {
                    s::BindingSource::Glob
                } else {
                    s::BindingSource::Import
                },
                import.node,
            ))
        }
    }
}
