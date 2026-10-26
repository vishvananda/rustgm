//! Actual resolved bindings and clauses produce complete version-two coordinates once.
use super::{Declarations, DefId, DefKind, Namespace, Report, Stage, g, s};
use crate::{
    resolve::{
        impl_restrictions,
        model::{Access, Binding, Origin, Target},
        unit::NodeKey,
    },
    support::crates::{Namespace as WireNamespace, owner_catalog::DeclarationRef},
    syntax::ast::Kind,
};

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
    ) -> Result<s::ModuleContractRows, &'static str> {
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
        let mut result = s::ModuleContractRows {
            namespace: s::ModuleNamespace {
                root,
                modules: Vec::new(),
                bindings: Vec::new(),
            },
            domains: Vec::new(),
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
            let actual = DefId(u32::try_from(actual).map_err(|_| "module declaration overflow")?);
            let module = impl_restrictions::module(&report.package, &report.resolution, actual)?;
            let row = self.declaration(actual)?;
            if index.rows[module.0 as usize].replace(row).is_some() {
                return Err("module policy has duplicate actual modules");
            }
            let site = definition.node.map_or(Ok(0), |node| self.site(row, node))?;
            self.reserve(4 * size_of::<s::ModuleRow>())?;
            result.namespace.modules.push(s::ModuleRow {
                declaration: row,
                declaration_site: site,
            });
        }
        self.module_bindings(report, &index, &mut result.namespace)?;
        self.trait_domains(report, facts, &index, &mut result.domains)?;
        self.scratch(
            result
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
        let ordered = result.canonicalize(&mut work);
        let delta = work.steps.saturating_sub(self.work().steps);
        self.charge(delta)?;
        ordered?;
        Ok(result)
    }
    fn module_bindings(
        &mut self,
        report: &Report,
        index: &Modules,
        names: &mut s::ModuleNamespace,
    ) -> Result<(), &'static str> {
        for module in &names.modules {
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
                let namespace = match namespace {
                    Namespace::Type => WireNamespace::Type,
                    Namespace::Value => WireNamespace::Value,
                    _ => return Err("module contract namespace is unsupported"),
                };
                let visibility = match binding.access {
                    Access::Public => s::ModuleVisibility::Public,
                    Access::Within(module) => s::ModuleVisibility::Within(index.local(module)?),
                    Access::ExternalPrivate => {
                        return Err("foreign private module binding is unsupported");
                    }
                };
                let (source, node) = binding_origin(report, scope, binding)?;
                let source_site = self.site(module.declaration, node)?;
                let name = report.package.names.get(name);
                self.charge(name.len())?;
                self.reserve(4 * (size_of::<s::ModuleBindingRow>() + name.len()))?;
                names.bindings.push(s::ModuleBindingRow {
                    module: module.declaration,
                    name: name.to_owned(),
                    namespace,
                    target: self.address(binding.target)?,
                    visibility,
                    source,
                    source_site,
                });
            }
        }
        Ok(())
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
) -> Result<(s::BindingSource, NodeKey), &'static str> {
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
