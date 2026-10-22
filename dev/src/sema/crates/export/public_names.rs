//! Traverse public module identities once; aliases retain their defining targets.
use super::{DefId, DefKind, Error, Report, error, records};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::model::{Access, Origin, ScopeId},
    support::crates::{
        CrateIdentity, ExternalDef, ExternalKind, PublicBinding, PublicModule, PublicNames,
    },
};

pub(super) struct Plan {
    pub owned: Vec<(DefId, ExternalDef)>,
    pub names: Option<PublicNames>,
}

pub(super) fn collect(report: &Report, identity: &CrateIdentity) -> Result<Plan, Error> {
    let imported: HashMap<_, _> = report
        .resolution
        .crates
        .as_deref()
        .into_iter()
        .flat_map(|imports| imports.definitions().map(|(external, id)| (id, external)))
        .collect();
    let root = report.resolution.module_scopes[0];
    let mut work = vec![(root, Vec::new())];
    let mut seen = HashSet::from_iter([root]);
    let mut targets = HashMap::<DefId, ExternalDef>::default();
    let mut owned = Vec::new();
    let mut modules = Vec::new();
    let mut explicit = false;
    let mut visits = 0usize;
    while let Some((scope, path)) = work.pop() {
        let mut bindings = Vec::new();
        for (name, _, binding) in report.resolution.bindings.scope(scope) {
            if binding.access != Access::Public {
                continue;
            }
            if binding.ambiguous {
                return Err(error(
                    "ambiguous public bindings are outside the metadata profile",
                ));
            }
            visits += 1;
            if visits > 100_000 {
                return Err(error("metadata public namespace exceeds profile budget"));
            }
            explicit |= binding.origin != Origin::Direct;
            let definition = binding.target;
            let target = if let Some(target) = targets.get(&definition) {
                target.clone()
            } else {
                let target = imported.get(&definition).map_or_else(
                    || local_target(report, identity, definition, root),
                    |target| Ok((**target).clone()),
                )?;
                if target.owner() == identity {
                    if target.kind() == ExternalKind::Module {
                        let members = report.resolution.definitions[definition.0 as usize]
                            .members
                            .ok_or_else(|| error("missing public module scope"))?;
                        if seen.insert(members) {
                            work.push((members, target.path().to_vec()));
                        }
                    } else {
                        owned.push((definition, target.clone()));
                    }
                }
                targets.insert(definition, target.clone());
                target
            };
            bindings.push(
                PublicBinding::new(report.package.names.get(name).to_owned(), target)
                    .map_err(error)?,
            );
        }
        explicit |= scope != root && bindings.is_empty();
        modules.push(PublicModule::new(path, bindings).map_err(error)?);
    }
    let names = if explicit {
        Some(PublicNames::new(modules).map_err(error)?)
    } else {
        None
    };
    Ok(Plan { owned, names })
}

fn local_target(
    report: &Report,
    identity: &CrateIdentity,
    definition: DefId,
    root: ScopeId,
) -> Result<ExternalDef, Error> {
    let entry = &report.resolution.definitions[definition.0 as usize];
    let kind = match entry.kind {
        DefKind::Module => ExternalKind::Module,
        DefKind::Function => ExternalKind::Function,
        DefKind::Const => ExternalKind::Constant,
        DefKind::Struct => ExternalKind::Record,
        DefKind::Static => ExternalKind::Static,
        _ => {
            return Err(error(
                "exposed declaration is outside the scalar metadata profile",
            ));
        }
    };
    if kind == ExternalKind::Module && entry.members == Some(root) {
        return ExternalDef::new(identity.clone(), Vec::new(), kind).map_err(error);
    }
    records::path(report, identity, definition, kind)
}
