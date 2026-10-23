//! Scope and import ownership comes from the resolved graph, never name spelling.
use super::{Draft, PolicyId};
use crate::{
    resolve::{
        model::{DefKind, Resolution, ScopeId},
        package::{Error, Package},
        unit::NodeKey,
    },
    support::hash::HashSet,
    syntax::{ast::Kind, item::ItemKind},
};

pub(super) struct Owners {
    pub declarations: Vec<Option<PolicyId>>,
    pub imports: Vec<Option<PolicyId>>,
}

pub(super) fn collect(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft,
) -> Result<Owners, Error> {
    let count = facts
        .scopes
        .len()
        .saturating_add(facts.definitions.len())
        .saturating_add(facts.imports.len());
    draft.charge(
        count,
        count.saturating_mul(4 * size_of::<Option<PolicyId>>()),
    )?;
    let mut scopes = vec![None; facts.scopes.len()];
    let mut used = HashSet::default();
    modules(package, facts, draft, &mut scopes, &mut used)?;
    definition_scopes(package, facts, draft, &mut scopes, &mut used)?;
    // Lexical scope IDs are append-only and parent-first. Module scopes were
    // seeded separately because their lexical parent is intentionally absent.
    for (index, scope) in facts.scopes.iter().enumerate() {
        draft.charge(1, 0)?;
        if let Some(parent) = scope.parent {
            if parent.0 as usize >= index {
                return Err(Error::unlocated(
                    "library scope parent is not earlier than its child",
                ));
            }
            if scopes[index].is_none() {
                scopes[index] = draft.inherit(scopes[parent.0 as usize]);
            }
        }
    }
    draft.charge(facts.definitions.len(), 0)?;
    let declarations = facts
        .definitions
        .iter()
        .enumerate()
        .map(|(index, definition)| {
            if definition.kind == DefKind::Module {
                if definition.node.is_none()
                    && facts.crate_root.is_none_or(|root| root.0 as usize != index)
                {
                    return None;
                }
                return definition
                    .members
                    .and_then(|scope| scopes[scope.0 as usize]);
            }
            if definition.kind != DefKind::Function {
                return None;
            }
            let key = definition.node?;
            let explicit = draft.attributes.get(&key).copied();
            explicit.or_else(|| draft.inherit(scopes[definition.scope.0 as usize]))
        })
        .collect();
    import_attributes(package, draft, &mut used)?;
    let imports = facts
        .imports
        .iter()
        .map(|import| {
            draft
                .import_attributes
                .get(&import.node)
                .copied()
                .or_else(|| draft.inherit(scopes[import.scope.0 as usize]))
        })
        .collect();
    if let Some(policy) = draft
        .policies
        .iter()
        .find(|policy| !used.contains(&policy.origin))
    {
        return Err(error(
            package,
            policy.origin,
            "unsupported library stability annotation owner",
        ));
    }
    Ok(Owners {
        declarations,
        imports,
    })
}

fn definition_scopes(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft,
    scopes: &mut [Option<PolicyId>],
    used: &mut HashSet<NodeKey>,
) -> Result<(), Error> {
    draft.charge(facts.definitions.len(), 0)?;
    for definition in &facts.definitions {
        let Some(key) = definition.node else {
            continue;
        };
        let unit = package.unit(key.file);
        let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
            continue;
        };
        let item = unit.configured.ast.items[index as usize];
        match item.kind {
            ItemKind::Module { .. } | ItemKind::MacroRules { .. } => {}
            ItemKind::Function {
                flags,
                body: Some(_),
                ..
            } if !flags.constant
                && !flags.asynchronous
                && item.generics.start == item.generics.end
                && item.where_clause.start == item.where_clause.end =>
            {
                if definition.kind != DefKind::Function {
                    continue;
                }
                let module = facts.scopes[definition.scope.0 as usize].module;
                let scope = *facts.node_scopes.get(&(module, key.node)).ok_or_else(|| {
                    error(package, key, "library function has no actual owner scope")
                })?;
                if let Some(&policy) = draft.attributes.get(&key) {
                    used.insert(key);
                    scopes[scope.0 as usize] = Some(policy);
                }
            }
            _ => {
                return Err(error(
                    package,
                    key,
                    "this local library stability profile requires ordinary functions and modules",
                ));
            }
        }
    }
    Ok(())
}

fn modules(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft,
    scopes: &mut [Option<PolicyId>],
    used: &mut HashSet<NodeKey>,
) -> Result<(), Error> {
    for (index, module) in package.modules.iter().enumerate() {
        draft.charge(1, 0)?;
        let root = underlying(
            package,
            NodeKey {
                file: module.file,
                node: module.root,
            },
            draft,
        )?;
        let inner = draft.attributes.get(&root).copied();
        let declaration = module
            .declaration
            .map(|key| underlying(package, key, draft))
            .transpose()?;
        let outer = declaration
            .filter(|key| *key != root)
            .and_then(|key| draft.attributes.get(&key))
            .copied();
        if inner.is_some() && outer.is_some() {
            return Err(error(
                package,
                root,
                "duplicate outer and inner library stability",
            ));
        }
        if inner.is_some() {
            used.insert(root);
        }
        if outer.is_some()
            && let Some(key) = declaration
        {
            used.insert(key);
        }
        let parent = match module.parent {
            Some(parent) if (parent.0 as usize) < index => {
                draft.inherit(scopes[facts.module_scopes[parent.0 as usize].0 as usize])
            }
            Some(_) => {
                return Err(error(
                    package,
                    root,
                    "library module parent is not earlier than its child",
                ));
            }
            None => None,
        };
        let effective = inner.or(outer).or(parent);
        if index == 0 && effective.is_none() {
            return Err(error(
                package,
                root,
                "crate has missing library stability attribute",
            ));
        }
        scopes[facts.module_scopes[index].0 as usize] = effective;
    }
    Ok(())
}

fn underlying(package: &Package, mut key: NodeKey, draft: &mut Draft) -> Result<NodeKey, Error> {
    // The module root keeps its attribute wrapper; the existing claims pass
    // indexes the underlying owner. Follow only these retained wrapper edges.
    while let Kind::Attributed { value, .. } =
        package.unit(key.file).configured.ast.node(key.node).kind
    {
        draft.charge(1, 0)?;
        key.node = value;
    }
    Ok(key)
}

fn import_attributes(
    package: &Package,
    draft: &mut Draft,
    used: &mut HashSet<NodeKey>,
) -> Result<(), Error> {
    // Only annotated import groups need an auxiliary leaf lookup. A group is
    // expanded once; each actual Import still owns its effective policy.
    draft.charge(
        draft.attributes.len(),
        draft.attributes.len() * size_of::<(NodeKey, PolicyId)>(),
    )?;
    let entries: Vec<_> = draft
        .attributes
        .iter()
        .map(|(&key, &id)| (key, id))
        .collect();
    let mut pending = Vec::new();
    for (key, id) in entries {
        let unit = package.unit(key.file);
        let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
            continue;
        };
        let ItemKind::Use(tree) = unit.configured.ast.items[index as usize].kind else {
            continue;
        };
        used.insert(key);
        pending.push(tree);
        while let Some(node) = pending.pop() {
            draft.charge(1, 4 * size_of::<(NodeKey, PolicyId)>())?;
            let Kind::UseTree(index) = unit.configured.ast.node(node).kind else {
                return Err(error(
                    package,
                    key,
                    "library import has an invalid retained tree",
                ));
            };
            let children = unit.configured.ast.use_trees[index as usize].children;
            if children.start == children.end {
                draft.import_attributes.insert(
                    NodeKey {
                        file: key.file,
                        node,
                    },
                    id,
                );
            } else {
                draft.charge(
                    (children.end - children.start) as usize,
                    4 * (children.end - children.start) as usize
                        * size_of::<crate::syntax::ast::NodeId>(),
                )?;
                pending.extend_from_slice(
                    &unit.configured.ast.edges[children.start as usize..children.end as usize],
                );
            }
        }
    }
    Ok(())
}

pub(super) fn error(package: &Package, key: NodeKey, message: &'static str) -> Error {
    Error::at(
        key.file,
        package
            .unit(key.file)
            .configured
            .ast
            .node(key.node)
            .span
            .start,
        message,
    )
}

pub(super) fn reachable(
    package: &Package,
    facts: &Resolution,
    owners: &Owners,
    draft: &mut Draft,
) -> Result<(), Error> {
    use crate::resolve::model::Access;
    let root = *facts
        .module_scopes
        .first()
        .ok_or_else(|| Error::unlocated("library root scope is absent"))?;
    let mut seen = HashSet::from_iter([root]);
    let mut pending = vec![root];
    while let Some(scope) = pending.pop() {
        draft.charge(1, 4 * size_of::<ScopeId>())?;
        for (_, _, binding) in facts.bindings.scope(scope) {
            draft.charge(1, 0)?;
            if binding.access != Access::Public {
                continue;
            }
            if binding.ambiguous {
                return Err(Error::unlocated("ambiguous public library binding"));
            }
            let definition = &facts.definitions[binding.target.0 as usize];
            let Some(key) = definition.node else {
                continue;
            };
            if owners.declarations[binding.target.0 as usize].is_none() {
                return Err(error(
                    package,
                    key,
                    "public declaration has missing library stability attribute",
                ));
            }
            if definition.kind == DefKind::Module
                && let Some(members) = definition.members
                && seen.insert(members)
            {
                pending.push(members);
            }
        }
    }
    for (index, import) in facts.imports.iter().enumerate() {
        draft.charge(1, 0)?;
        if import.access == Access::Public
            && seen.contains(&import.scope)
            && owners.imports[index].is_none()
        {
            return Err(error(
                package,
                import.node,
                "public import has missing library stability attribute",
            ));
        }
    }
    Ok(())
}
