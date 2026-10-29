//! Validate a complete immutable dependency graph before allocating local identities.
use super::{
    CrateIdentity, Interface, MAX_COMPONENTS, MAX_DEPENDENCIES, MAX_FUNCTIONS, MAX_NAME_BYTES,
};
use crate::support::hash::HashMap;
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
pub struct Dependency {
    alias: String,
    identity: CrateIdentity,
}
impl Dependency {
    /// Bind a validated semantic crate alias to an exact artifact identity.
    /// # Errors
    /// Rejects malformed aliases and reserved crate-root names.
    pub fn new(alias: String, identity: CrateIdentity) -> Result<Self, &'static str> {
        super::identity::validate_component(&alias)?;
        if matches!(
            alias.as_str(),
            "self" | "Self" | "super" | "crate" | "core" | "std"
        ) {
            return Err("unsupported external crate alias");
        }
        Ok(Self { alias, identity })
    }
    #[must_use]
    pub fn alias(&self) -> &str {
        &self.alias
    }
    #[must_use]
    pub const fn identity(&self) -> &CrateIdentity {
        &self.identity
    }
    /// Cost only for the owned alias entry; crate identities share their Arcs.
    pub(crate) fn storage_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(self.alias.capacity())
    }
}
#[derive(Debug)]
pub struct Dependencies {
    artifacts: Vec<Interface>,
    aliases: Vec<Dependency>,
    edges: usize,
}
impl Dependencies {
    /// Select one actual supported source provider, independently of aliases.
    /// Initial admission searches once; clients retain the exact inventory/index.
    /// # Errors
    /// Rejects missing, legacy, mismatched and unsupported provider descriptors.
    pub fn source_provider_index(&self, provider: &CrateIdentity) -> Result<u32, &'static str> {
        let index = self
            .artifacts
            .binary_search_by(|artifact| artifact.identity().cmp(provider))
            .map_err(|_| "source client provider metadata is missing")?;
        let descriptor = self.artifacts[index]
            .source_descriptor()
            .ok_or("source client requires a complete supported source provider payload")?;
        if descriptor.identity() != provider
            || descriptor.provider() != provider
            || descriptor.edition() != crate::support::edition::Edition::Rust2024
        {
            return Err("source client provider descriptor does not match the requested profile");
        }
        if !descriptor.profile().ordinary_functions() {
            match descriptor.profile() {
                super::generics::source::Profile::Source2024ModuleContracts
                | super::generics::source::Profile::Source2024ModuleAvailability
                | super::generics::source::Profile::Source2024ModuleFeaturePredicates
                | super::generics::source::Profile::Source2024ModuleDefinitions
                | super::generics::source::Profile::Source2024ModuleCompatibility
                | super::generics::source::Profile::Source2024ModuleCallables
                | super::generics::source::Profile::Source2024ModuleSafety => {
                    descriptor.module_contracts()?;
                }
                _ => return Err("source client provider profile is unsupported"),
            }
        }
        u32::try_from(index).map_err(|_| "source provider artifact index overflow")
    }
    /// Canonicalize once and validate missing/duplicate identities and cycles.
    /// # Errors
    /// Rejects incomplete graphs, ambiguous aliases and bounded table exhaustion.
    pub fn new(
        mut artifacts: Vec<Interface>,
        mut aliases: Vec<Dependency>,
    ) -> Result<Self, &'static str> {
        if artifacts.len() > MAX_DEPENDENCIES || aliases.len() > MAX_DEPENDENCIES {
            return Err("crate graph exceeds profile budget");
        }
        artifacts.sort_unstable_by(|left, right| left.identity().cmp(right.identity()));
        aliases.sort_unstable_by(|left, right| left.alias.cmp(&right.alias));
        if artifacts
            .windows(2)
            .any(|pair| pair[0].identity() == pair[1].identity())
            || aliases
                .windows(2)
                .any(|pair| pair[0].alias == pair[1].alias)
        {
            return Err("duplicate external crate identity or alias");
        }
        let indices: HashMap<_, _> = artifacts
            .iter()
            .enumerate()
            .map(|(index, artifact)| (artifact.identity(), index))
            .collect();
        for alias in &aliases {
            if !indices.contains_key(&alias.identity) {
                return Err("external alias has no metadata artifact");
            }
        }
        let (atoms, names) = artifacts
            .iter()
            .fold((0usize, 0usize), |(atoms, names), artifact| {
                let (next_atoms, next_names) = artifact.budget();
                (
                    atoms.saturating_add(next_atoms),
                    names.saturating_add(next_names),
                )
            });
        let names = aliases.iter().fold(names, |bytes, alias| {
            bytes.saturating_add(alias.alias.len())
        });
        if atoms > MAX_COMPONENTS || names > MAX_NAME_BYTES {
            return Err("crate graph payload exceeds profile budget");
        }
        let edges = validate_graph(&artifacts, &indices)?;
        super::public_names::exposure::validate(&artifacts, &indices)?;
        Ok(Self {
            artifacts,
            aliases,
            edges,
        })
    }
    #[must_use]
    pub fn artifacts(&self) -> &[Interface] {
        &self.artifacts
    }
    #[must_use]
    pub fn aliases(&self) -> &[Dependency] {
        &self.aliases
    }
    /// Number of graph edges checked and processed once during construction.
    #[must_use]
    pub const fn edges(&self) -> usize {
        self.edges
    }
}
fn validate_graph(
    artifacts: &[Interface],
    indices: &HashMap<&CrateIdentity, usize>,
) -> Result<usize, &'static str> {
    let mut incoming = vec![0usize; artifacts.len()];
    let mut users = vec![Vec::new(); artifacts.len()];
    let mut edges = 0usize;
    let mut functions = 0usize;
    for (index, artifact) in artifacts.iter().enumerate() {
        functions = functions
            .saturating_add(artifact.functions().len())
            .saturating_add(
                artifact
                    .generic_payload()
                    .map_or(0, |payload| match payload.view() {
                        super::generics::PayloadView::Legacy(rows) => rows.functions.len(),
                        super::generics::PayloadView::Source(rows) => {
                            rows.descriptor().rows().functions.len()
                        }
                    }),
            )
            .saturating_add(artifact.constants().len())
            .saturating_add(artifact.nominals().map_or(0, |n| n.records().len()));
        edges = edges.saturating_add(artifact.dependencies().len());
        if functions > MAX_FUNCTIONS || edges > MAX_COMPONENTS {
            return Err("crate graph contents exceed profile budget");
        }
        for dependency in artifact.dependencies() {
            let &target = indices
                .get(dependency)
                .ok_or("missing crate dependency metadata")?;
            incoming[index] += 1;
            users[target].push(index);
        }
    }
    let mut ready: VecDeque<_> = incoming
        .iter()
        .enumerate()
        .filter_map(|(index, &count)| (count == 0).then_some(index))
        .collect();
    let mut completed = 0;
    while let Some(index) = ready.pop_front() {
        completed += 1;
        for &user in &users[index] {
            incoming[user] -= 1;
            if incoming[user] == 0 {
                ready.push_back(user);
            }
        }
    }
    if completed != artifacts.len() {
        return Err("cyclic crate dependency metadata");
    }
    Ok(edges)
}
