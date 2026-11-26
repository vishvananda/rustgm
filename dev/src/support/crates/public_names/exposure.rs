//! Dependency exposure is indexed once per referenced artifact before import.
use super::{
    ExternalDef, ExternalKind, Interface, PublicBinding, PublicModule, owned_public_definitions,
};
use crate::support::crates::CrateIdentity;
use crate::support::hash::{HashMap, HashSet};
mod dependency_applications;
use crate::support::crates::generics::{self as g, source as s};

struct Exposed<'a> {
    definitions: HashSet<&'a ExternalDef>,
    modules: HashSet<&'a [String]>,
}
impl<'a> Exposed<'a> {
    fn new(artifact: &'a Interface) -> Self {
        if let Some(names) = artifact.public_names() {
            return Self {
                definitions: names
                    .modules()
                    .iter()
                    .flat_map(PublicModule::bindings)
                    .map(PublicBinding::target)
                    .filter(|d| d.owner() == artifact.identity())
                    .collect(),
                modules: names.modules().iter().map(PublicModule::path).collect(),
            };
        }
        let definitions: HashSet<_> = owned_public_definitions(artifact).collect();
        let mut modules = HashSet::from_iter([&[][..]]);
        for definition in &definitions {
            for length in 1..definition.path().len() {
                modules.insert(&definition.path()[..length]);
            }
        }
        Self {
            definitions,
            modules,
        }
    }
    fn contains(&self, target: &ExternalDef) -> bool {
        if target.kind() == ExternalKind::Module {
            self.modules.contains(target.path())
        } else {
            self.definitions.contains(target)
        }
    }
}
pub(crate) fn validate(
    artifacts: &[Interface],
    indices: &HashMap<&CrateIdentity, usize>,
) -> Result<(), &'static str> {
    let mut exposed = HashMap::default();
    for artifact in artifacts {
        let Some(names) = artifact.public_names() else {
            continue;
        };
        for target in names
            .modules()
            .iter()
            .flat_map(PublicModule::bindings)
            .map(PublicBinding::target)
        {
            if target.owner() == artifact.identity() {
                continue;
            }
            let &index = indices
                .get(target.owner())
                .ok_or("public metadata target owner has no artifact")?;
            let entry = exposed
                .entry(index)
                .or_insert_with(|| Exposed::new(&artifacts[index]));
            if !entry.contains(target) {
                return Err("public metadata dependency target is private or missing");
            }
        }
    }
    Ok(())
}

/// Authenticate typed imports against the reachable namespace of the actual
/// defining artifact. Packet-local sealing checks the importing module.
pub(crate) fn validate_typed(
    artifacts: &[Interface],
    indices: &HashMap<&CrateIdentity, usize>,
) -> Result<(), &'static str> {
    use crate::support::crates::Namespace;
    let mut targets = TypedTargets {
        artifacts,
        indices,
        exposed: HashMap::default(),
    };
    for artifact in artifacts {
        let Some(descriptor) = artifact
            .source_descriptor()
            .filter(|descriptor| descriptor.profile().bootstrap_bindings())
        else {
            continue;
        };
        for binding in &descriptor.module_contracts()?.rows().namespace.bindings {
            if binding.target.artifact.0 == 0 {
                continue;
            }
            let flag = match binding.namespace {
                Namespace::Type => 1,
                Namespace::Value => 2,
            };
            targets.target(artifact, binding.target, flag)?;
        }
        dependency_applications::check(&mut targets, artifact, descriptor.rows())?;
    }
    Ok(())
}

struct TypedTargets<'a> {
    artifacts: &'a [Interface],
    indices: &'a HashMap<&'a CrateIdentity, usize>,
    exposed: HashMap<usize, TypedExports<'a>>,
}
struct TypedExports<'a> {
    names: Vec<u8>,
    rows: s::ValidatedView<'a>,
    slots: HashMap<crate::support::crates::owner_catalog::DeclarationRef, g::Range>,
}
impl<'a> TypedTargets<'a> {
    fn target(
        &mut self,
        artifact: &Interface,
        address: s::DeclarationAddress,
        flag: u8,
    ) -> Result<&TypedExports<'a>, &'static str> {
        let identity = artifact.declaration_owner(address.artifact)?;
        let &index = self
            .indices
            .get(identity)
            .ok_or("typed namespace target has no dependency artifact")?;
        let exports = match self.exposed.entry(index) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(typed_exports(&self.artifacts[index])?)
            }
        };
        if exports
            .names
            .get(address.declaration.0 as usize)
            .is_none_or(|namespaces| namespaces & flag == 0)
        {
            return Err(
                "typed namespace dependency target is private, absent or in another namespace",
            );
        }
        Ok(exports)
    }
}

fn typed_exports(artifact: &Interface) -> Result<TypedExports<'_>, &'static str> {
    use crate::support::crates::{
        Namespace,
        generics::source::{
            ArtifactRef, DefinitionKind, Exhaustiveness, HeaderRef, ModuleVisibility,
        },
    };
    let descriptor = artifact
        .source_descriptor()
        .filter(|descriptor| descriptor.profile().bootstrap_bindings())
        .ok_or("typed namespace dependency requires its ordinary typed artifact")?;
    let rows = descriptor.rows();
    let modules = descriptor.module_contracts()?;
    let namespace = &modules.rows().namespace;
    let mut exposed = vec![0; rows.declarations.len()];
    let mut pending = vec![namespace.root];
    exposed[namespace.root.0 as usize] = 1;
    while let Some(module) = pending.pop() {
        let start = namespace
            .bindings
            .partition_point(|entry| entry.module.0 < module.0);
        let end = namespace
            .bindings
            .partition_point(|entry| entry.module.0 <= module.0);
        for binding in &namespace.bindings[start..end] {
            if binding.visibility != ModuleVisibility::Public
                || binding.target.artifact != ArtifactRef(0)
            {
                continue;
            }
            let row = binding.target.declaration;
            if let Some(HeaderRef::Constructor(index)) = rows.header(row)?
                && rows.nominals[index as usize].exhaustiveness == Exhaustiveness::Open
            {
                continue;
            }
            let flag = match binding.namespace {
                Namespace::Type => 1,
                Namespace::Value => 2,
            };
            let exposed = &mut exposed[row.0 as usize];
            if *exposed == 0
                && rows.declarations[row.0 as usize].definition.kind == DefinitionKind::Module
            {
                pending.push(row);
            }
            *exposed |= flag;
        }
    }
    Ok(TypedExports {
        names: exposed,
        rows,
        slots: rows
            .scopes
            .iter()
            .map(|scope| (scope.owner, scope.declared))
            .collect(),
    })
}
