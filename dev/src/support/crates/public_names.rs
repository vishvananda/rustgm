//! Public bindings point to defining identities; module aliases share one table.
pub(super) mod exposure;
use super::{
    ExternalDef, ExternalKind, Interface, MAX_COMPONENTS, MAX_FUNCTIONS, MAX_NAME_BYTES, Namespace,
};
use crate::support::hash::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicBinding {
    name: String,
    target: ExternalDef,
}
impl PublicBinding {
    /// Retain an exported spelling independently of the target's canonical path.
    /// # Errors
    /// Rejects noncanonical Rust identifiers.
    pub fn new(name: String, target: ExternalDef) -> Result<Self, &'static str> {
        super::identity::validate_component(&name)?;
        Ok(Self { name, target })
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn target(&self) -> &ExternalDef {
        &self.target
    }
    fn key(&self) -> (&str, Namespace) {
        (&self.name, self.target.namespace())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicModule {
    path: Vec<String>,
    bindings: Vec<PublicBinding>,
}
impl PublicModule {
    /// Canonicalize one module; its empty path denotes the artifact root.
    /// # Errors
    /// Rejects malformed paths, duplicate namespace bindings and oversized tables.
    pub fn new(path: Vec<String>, mut bindings: Vec<PublicBinding>) -> Result<Self, &'static str> {
        if path.len() > 64 || bindings.len() > MAX_FUNCTIONS {
            return Err("public metadata module exceeds profile budget");
        }
        for component in &path {
            super::identity::validate_component(component)?;
        }
        bindings.sort_unstable_by(|left, right| left.key().cmp(&right.key()));
        if bindings
            .windows(2)
            .any(|pair| pair[0].key() == pair[1].key())
        {
            return Err("duplicate public metadata namespace binding");
        }
        Ok(Self { path, bindings })
    }
    #[must_use]
    pub fn path(&self) -> &[String] {
        &self.path
    }
    #[must_use]
    pub fn bindings(&self) -> &[PublicBinding] {
        &self.bindings
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PublicNames {
    modules: Vec<PublicModule>,
    atoms: usize,
    names: usize,
}
impl PublicNames {
    pub(super) fn allocation_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(crate::support::retained::vector(&self.modules))
            .saturating_add(
                self.modules
                    .iter()
                    .map(|module| {
                        crate::support::retained::vector(&module.path)
                            .saturating_add(crate::support::retained::vector(&module.bindings))
                            .saturating_add(
                                module
                                    .path
                                    .iter()
                                    .map(String::capacity)
                                    .fold(0_usize, usize::saturating_add),
                            )
                            .saturating_add(
                                module
                                    .bindings
                                    .iter()
                                    .map(|binding| binding.name.capacity())
                                    .fold(0_usize, usize::saturating_add),
                            )
                    })
                    .fold(0_usize, usize::saturating_add),
            )
    }
    /// Store each canonical module once, without unfolding alias paths.
    /// # Errors
    /// Rejects missing roots, duplicate modules and combined storage exhaustion.
    pub fn new(mut modules: Vec<PublicModule>) -> Result<Self, &'static str> {
        if modules.is_empty() || modules.len() > MAX_FUNCTIONS {
            return Err("invalid public metadata module count");
        }
        modules.sort_unstable_by(|left, right| left.path.cmp(&right.path));
        if !modules[0].path.is_empty()
            || modules.windows(2).any(|pair| pair[0].path == pair[1].path)
        {
            return Err("missing root or duplicate public metadata module");
        }
        let mut atoms = modules.len();
        let mut names = 0usize;
        for module in &modules {
            atoms = atoms.saturating_add(module.path.len());
            names = names.saturating_add(module.path.iter().map(String::len).sum::<usize>());
            for binding in &module.bindings {
                atoms = atoms.saturating_add(1 + binding.target.path().len());
                names = names
                    .saturating_add(binding.name.len())
                    .saturating_add(binding.target.path().iter().map(String::len).sum::<usize>());
            }
            if atoms > MAX_COMPONENTS || names > MAX_NAME_BYTES {
                return Err("public metadata names exceed profile budget");
            }
        }
        Ok(Self {
            modules,
            atoms,
            names,
        })
    }
    #[must_use]
    pub fn modules(&self) -> &[PublicModule] {
        &self.modules
    }
    pub(super) const fn budget(&self) -> (usize, usize) {
        (self.atoms, self.names)
    }
    pub(super) fn validate(&self, interface: &Interface) -> Result<(), &'static str> {
        let modules: HashMap<_, _> = self
            .modules
            .iter()
            .enumerate()
            .map(|(index, module)| (module.path.as_slice(), index))
            .collect();
        let definitions: HashSet<_> = owned_public_definitions(interface).collect();
        let dependencies: HashSet<_> = interface.dependencies().iter().collect();
        let mut exposed = HashSet::default();
        let mut reached = vec![false; self.modules.len()];
        reached[0] = true;
        let mut pending = vec![0];
        while let Some(index) = pending.pop() {
            for binding in &self.modules[index].bindings {
                let target = &binding.target;
                if target.owner() != interface.identity() {
                    if !dependencies.contains(target.owner()) {
                        return Err("public metadata target has no declared dependency");
                    }
                } else if target.kind() == ExternalKind::Module {
                    let &next = modules
                        .get(target.path())
                        .ok_or("public metadata target has no module table")?;
                    if !reached[next] {
                        reached[next] = true;
                        pending.push(next);
                    }
                } else if !definitions.contains(target) {
                    return Err("public metadata target has no exposed owned declaration");
                } else {
                    exposed.insert(target);
                }
            }
        }
        if reached.contains(&false) || exposed.len() != definitions.len() {
            return Err("unreachable public metadata module or declaration");
        }
        Ok(())
    }
}
pub(super) fn owned_public_definitions(
    interface: &Interface,
) -> impl Iterator<Item = &ExternalDef> {
    interface
        .functions()
        .iter()
        .map(super::Function::definition)
        .chain(
            interface
                .generic_payload()
                .into_iter()
                .filter_map(|payload| match payload.view() {
                    super::generics::PayloadView::Legacy(rows) => Some(&rows.declarations),
                    super::generics::PayloadView::Source(_) => None,
                })
                .flatten()
                .filter_map(|declaration| match declaration {
                    super::generics::Declaration::Function {
                        identity,
                        public: true,
                    } => Some(identity),
                    super::generics::Declaration::Function { public: false, .. }
                    | super::generics::Declaration::TypeParameter { .. }
                    | super::generics::Declaration::ValueParameter { .. }
                    | super::generics::Declaration::Local { .. } => None,
                }),
        )
        .chain(
            interface
                .constants()
                .iter()
                .map(super::Constant::definition),
        )
        .chain(
            interface
                .records()
                .iter()
                .filter(|row| row.public())
                .map(super::Record::definition),
        )
        .chain(
            interface
                .storage()
                .into_iter()
                .flat_map(super::Storage::statics)
                .filter(|row| row.public())
                .map(super::Static::definition),
        )
}
