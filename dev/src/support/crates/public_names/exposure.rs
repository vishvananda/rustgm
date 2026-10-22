//! Dependency exposure is indexed once per referenced artifact before import.
use super::{
    ExternalDef, ExternalKind, Interface, PublicBinding, PublicModule, owned_public_definitions,
};
use crate::support::crates::CrateIdentity;
use std::collections::{HashMap, HashSet};

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
        let mut modules = HashSet::from([&[][..]]);
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
    let mut exposed = HashMap::new();
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
