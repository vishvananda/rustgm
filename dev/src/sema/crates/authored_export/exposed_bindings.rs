//! Capture actual resolved module bindings once; canonical paths are not identities.
use super::{DefId, DefKind, Namespace, Report, Stage, s};
use crate::support::crates::Namespace as WireNamespace;

impl Stage {
    pub(crate) fn namespace_columns(
        &mut self,
        report: &Report,
    ) -> Result<s::NamespaceColumns, &'static str> {
        let root = self.declaration(
            report
                .resolution
                .crate_root
                .ok_or("namespace source has no actual root declaration")?,
        )?;
        let mut names = s::NamespaceColumns {
            root,
            modules: Vec::new(),
            bindings: Vec::new(),
        };
        for (index, declaration) in report.resolution.definitions.iter().enumerate() {
            self.charge(1)?;
            if declaration.kind != DefKind::Module {
                continue;
            }
            let actual = DefId(u32::try_from(index).map_err(|_| "namespace declaration overflow")?);
            crate::resolve::impl_restrictions::module(&report.package, &report.resolution, actual)?;
            let module = self.declaration(actual)?;
            self.reserve(4 * size_of::<crate::support::crates::owner_catalog::DeclarationRef>())?;
            names.modules.push(module);
            let scope = declaration
                .members
                .ok_or("namespace module has no actual member scope")?;
            for (name, namespace, binding) in report.resolution.bindings.scope(scope) {
                self.charge(1)?;
                if binding.access != crate::resolve::model::Access::Public {
                    continue;
                }
                if binding.ambiguous {
                    return Err("namespace source binding is ambiguous");
                }
                let namespace = match namespace {
                    Namespace::Type => WireNamespace::Type,
                    Namespace::Value => WireNamespace::Value,
                    _ => return Err("namespace source binding requires a wider namespace profile"),
                };
                let spelling = report.package.names.get(name);
                self.charge(spelling.len())?;
                self.reserve(4 * (size_of::<s::ExportBinding>() + spelling.len()))?;
                let target = self.address(binding.target)?;
                names.bindings.push(s::ExportBinding {
                    module,
                    name: spelling.to_owned(),
                    namespace,
                    target,
                });
            }
        }
        Ok(names)
    }
}
