//! Scope and import ownership comes from the resolved graph, never name spelling.
mod availability_variants;
use super::{
    Draft, PolicyId, PolicyKind,
    constness::{ConstPolicies, Frame, Role},
};
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
    pub consts: Option<Box<ConstPolicies>>,
    pub fields: Option<Box<super::record_policies::Fields>>,
}

pub(super) fn collect(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft<'_>,
) -> Result<Owners, Error> {
    draft.finish_comptime()?;
    // Allocate each table on its own axis. Declarations are filled in their
    // paid pass; reserving their capacity does not visit their elements.
    draft.charge(
        facts.scopes.len().saturating_add(facts.imports.len()),
        facts
            .scopes
            .len()
            .saturating_mul(size_of::<Frame>())
            .saturating_add(
                facts
                    .definitions
                    .len()
                    .saturating_mul(size_of::<Option<PolicyId>>()),
            )
            .saturating_add(
                facts
                    .imports
                    .len()
                    .saturating_mul(size_of::<Option<PolicyId>>()),
            ),
    )?;
    let growth =
        crate::support::retained::Growth::map::<(NodeKey, bool), ()>(0, 0, draft.policies.len());
    draft.charge(growth.work, growth.bytes)?;
    let mut scopes = vec![Frame::default(); facts.scopes.len()];
    // Every used key names an actual captured policy. Reserve that finite upper
    // bound once so inserts cannot hide repeated hash-table growth.
    let mut used = HashSet::default();
    used.try_reserve(draft.policies.len())
        .map_err(|_| Error::unlocated("library policy ownership allocation failed"))?;
    modules(package, facts, draft, &mut scopes, &mut used)?;
    definition_scopes(package, facts, draft, &mut scopes, &mut used)?;
    // Lexical scope IDs are append-only and parent-first. Module scopes were
    // seeded separately because their lexical parent is intentionally absent.
    let mut has_const = false;
    for (index, scope) in facts.scopes.iter().enumerate() {
        draft.charge(1, 0)?;
        if let Some(parent) = scope.parent {
            if parent.0 as usize >= index {
                return Err(Error::unlocated(
                    "library scope parent is not earlier than its child",
                ));
            }
            let inherited = scopes[parent.0 as usize];
            scopes[index].inherit(draft, inherited);
        }
        has_const |=
            scopes[index].constant.is_some() || const_function(scope, &scopes[index], &scopes);
    }
    let mut owners = declaration_rows(package, facts, draft, &scopes, has_const)?;
    owners.fields = super::record_policies::collect(package, facts, &owners, draft, &mut used)?;
    import_attributes(package, draft, &mut used)?;
    owners.imports = facts
        .imports
        .iter()
        .map(|import| {
            draft
                .import_attributes
                .get(&import.node)
                .copied()
                .or_else(|| draft.inherit(scopes[import.scope.0 as usize].regular))
        })
        .collect();
    if let Some(policy) = draft.policies.iter().find(|policy| {
        policy.kind != PolicyKind::DefaultBody
            && !used.contains(&(policy.origin, policy.kind == PolicyKind::Const))
    }) {
        return Err(error(
            package,
            policy.origin,
            "unsupported library stability annotation owner",
        ));
    }
    Ok(owners)
}

fn declaration_rows(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft<'_>,
    scopes: &[Frame],
    has_const: bool,
) -> Result<Owners, Error> {
    draft.charge(facts.definitions.len(), 0)?;
    let mut declarations: Vec<Option<PolicyId>> = Vec::with_capacity(facts.definitions.len());
    if has_const {
        draft.charge(
            0,
            facts
                .definitions
                .len()
                .saturating_mul(size_of::<Option<PolicyId>>())
                .saturating_add(size_of::<ConstPolicies>()),
        )?;
    }
    let mut constants = has_const.then(|| Vec::with_capacity(facts.definitions.len()));
    let mut implicit = Vec::new();
    for (index, definition) in facts.definitions.iter().enumerate() {
        let scope = declaration_scope(package, facts, definition);
        let regular = if definition.kind == DefKind::Module {
            if definition.node.is_none()
                && facts.crate_root.is_none_or(|root| root.0 as usize != index)
            {
                declarations.push(None);
                if let Some(constants) = &mut constants {
                    constants.push(None);
                }
                continue;
            }
            definition
                .members
                .and_then(|id| scopes[id.0 as usize].regular)
        } else if definition.node.is_none()
            || (definition.kind.is_local() && definition.kind != DefKind::SelfType)
            || (definition.kind == DefKind::SelfType && scope.is_none())
        {
            None
        } else {
            definition
                .node
                .and_then(|key| draft.attribute(key, PolicyKind::Regular))
                .or_else(|| {
                    // Variants inherit stable as well as unstable enum policy.
                    // The resolved nominal owner precedes its actual children.
                    (definition.kind == DefKind::Variant)
                        .then(|| facts.scopes[definition.scope.0 as usize].nominal_self)
                        .flatten()
                        .and_then(|owner| declarations.get(owner.0 as usize).copied().flatten())
                })
                .or_else(|| draft.inherit(scopes[definition.scope.0 as usize].regular))
        };
        declarations.push(regular);
        if let Some(constants) = &mut constants {
            let policy = if definition.kind == DefKind::Module {
                definition
                    .members
                    .and_then(|id| scopes[id.0 as usize].constant)
            } else {
                scope.and_then(|id| scopes[id.0 as usize].constant)
            };
            constants.push(policy);
            if definition.kind == DefKind::Function
                && policy.is_none()
                && scope.is_some_and(|id| {
                    const_function(&facts.scopes[id.0 as usize], &scopes[id.0 as usize], scopes)
                })
            {
                draft.charge(1, 4 * size_of::<crate::resolve::model::DefId>())?;
                implicit.push(crate::resolve::model::DefId(
                    u32::try_from(declarations.len() - 1)
                        .map_err(|_| Error::unlocated("const function identity overflow"))?,
                ));
            }
        }
    }
    Ok(Owners {
        declarations,
        imports: Vec::new(),
        fields: None,
        consts: constants.map(|declarations| {
            Box::new(ConstPolicies {
                declarations,
                implicit,
                grants: None,
            })
        }),
    })
}
fn const_function(scope: &crate::resolve::model::Scope, frame: &Frame, scopes: &[Frame]) -> bool {
    matches!(frame.role, Role::Function(true))
        || matches!(frame.role, Role::Function(false))
            && scope.parent.is_some_and(|parent| {
                matches!(
                    scopes[parent.0 as usize].role,
                    Role::Trait(true) | Role::Impl(true) | Role::InherentImpl(true)
                )
            })
}

fn declaration_scope(
    package: &Package,
    facts: &Resolution,
    definition: &crate::resolve::model::Definition,
) -> Option<ScopeId> {
    let key = definition.node?;
    let module = facts.scopes[definition.scope.0 as usize].module;
    let Kind::Item(index) = package.unit(key.file).configured.ast.node(key.node).kind else {
        return None;
    };
    let item = package.unit(key.file).configured.ast.items[index as usize];
    // External aliases borrow their lexical scope; they do not own it.
    if matches!(definition.kind, DefKind::Constructor | DefKind::ExternCrate)
        || (definition.kind == DefKind::SelfType && !matches!(item.kind, ItemKind::Impl { .. }))
    {
        return None;
    }
    facts.node_scopes.get(&(module, key.node)).copied()
}

fn definition_scopes(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft<'_>,
    scopes: &mut [Frame],
    used: &mut HashSet<(NodeKey, bool)>,
) -> Result<(), Error> {
    draft.charge(facts.definitions.len(), 0)?;
    for definition in &facts.definitions {
        let Some(key) = definition.node else {
            continue;
        };
        if availability_variants::consume(package, facts, definition, draft, used)? {
            continue;
        }
        let Some(scope) = declaration_scope(package, facts, definition) else {
            continue;
        };
        let unit = package.unit(key.file);
        let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
            continue;
        };
        let item = unit.configured.ast.items[index as usize];
        let mut role = Role::of(item.kind);
        if matches!(role, Role::Function(false)) && draft.is_comptime(key)? {
            role = Role::Function(true);
        }
        if matches!(role, Role::Function(_))
            && draft.has_marker(key, super::FunctionMarker::NonConstMember)?
        {
            role = Role::NonConstMember;
        }
        let explicit = draft.attribute(key, PolicyKind::Const);
        // Direct function annotations require actual written constness (or a
        // checked comptime marker). Implicit member eligibility is a separate
        // fact; only the container's unstable policy is inherited.
        if explicit.is_some() && !role.accepts_const() {
            return Err(error(
                package,
                key,
                "const library stability requires a const function or an allowed declaration owner",
            ));
        }
        if matches!(
            item.kind,
            ItemKind::Module { .. }
                | ItemKind::MacroRules { .. }
                | ItemKind::DeclarativeMacro { .. }
        ) {
            if draft.attribute(key, PolicyKind::Regular).is_some() {
                used.insert((key, false));
            }
            continue;
        }
        if draft.attribute(key, PolicyKind::Regular).is_some() {
            used.insert((key, false));
        }
        if explicit.is_some() {
            used.insert((key, true));
        }
        scopes[scope.0 as usize] = Frame {
            regular: draft.attribute(key, PolicyKind::Regular),
            constant: explicit,
            role,
        };
    }
    Ok(())
}

fn modules(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft<'_>,
    scopes: &mut [Frame],
    used: &mut HashSet<(NodeKey, bool)>,
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
        let inner = draft.attribute(root, PolicyKind::Regular);
        let declaration = module
            .declaration
            .map(|key| underlying(package, key, draft))
            .transpose()?;
        let outer = declaration
            .filter(|key| *key != root)
            .and_then(|key| draft.attribute(key, PolicyKind::Regular));
        if inner.is_some() && outer.is_some() {
            return Err(error(
                package,
                root,
                "duplicate outer and inner library stability",
            ));
        }
        if inner.is_some() {
            used.insert((root, false));
        }
        if index == 0
            && let Some(constant) = draft.attribute(root, PolicyKind::Const)
        {
            used.insert((root, true));
            scopes[facts.module_scopes[index].0 as usize].constant = Some(constant);
        }
        if outer.is_some()
            && let Some(key) = declaration
        {
            used.insert((key, false));
        }
        let parent = match module.parent {
            Some(parent) if (parent.0 as usize) < index => {
                draft.inherit(scopes[facts.module_scopes[parent.0 as usize].0 as usize].regular)
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
        scopes[facts.module_scopes[index].0 as usize].regular = effective;
    }
    Ok(())
}

fn underlying(
    package: &Package,
    mut key: NodeKey,
    draft: &mut Draft<'_>,
) -> Result<NodeKey, Error> {
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
    draft: &mut Draft<'_>,
    used: &mut HashSet<(NodeKey, bool)>,
) -> Result<(), Error> {
    // Only annotated import groups need an auxiliary leaf lookup. A group is
    // expanded once; each actual Import still owns its effective policy.
    draft.charge(
        draft.attributes.len(),
        draft.attributes.len() * size_of::<(NodeKey, PolicyId)>(),
    )?;
    let mut entries = Vec::with_capacity(draft.attributes.len());
    entries.extend(draft.attributes.iter().filter_map(|(&(key, kind), &id)| {
        matches!(kind, PolicyKind::Regular | PolicyKind::Const).then_some((key, id))
    }));
    let mut pending = Vec::new();
    for (key, id) in entries {
        let unit = package.unit(key.file);
        let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
            continue;
        };
        let ItemKind::Use(tree) = unit.configured.ast.items[index as usize].kind else {
            continue;
        };
        let constant = draft
            .policies
            .get(id.get() - 1)
            .ok_or_else(|| error(package, key, "import policy lost its actual declaration"))?
            .kind
            == PolicyKind::Const;
        used.insert((key, constant));
        if constant {
            // Rust retains the annotation, but it supplies no target constness
            // or const API override through this import alias.
            continue;
        }
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
    draft: &mut Draft<'_>,
) -> Result<(), Error> {
    use crate::resolve::model::Access;
    let seen = super::visit_public_bindings(facts, draft, |target, draft| {
        let definition = &facts.definitions[target.0 as usize];
        let Some(key) = definition.node else {
            return Ok(());
        };
        if owners.declarations[target.0 as usize].is_none() {
            return Err(error(
                package,
                key,
                "public declaration has missing library stability attribute",
            ));
        }
        let unit = package.unit(key.file);
        let needs_const = match unit.configured.ast.node(key.node).kind {
            Kind::Item(index) => match unit.configured.ast.items[index as usize].kind {
                ItemKind::Function { flags, .. } => flags.constant || draft.is_comptime(key)?,
                ItemKind::Trait { constant, .. } => constant,
                _ => false,
            },
            _ => false,
        };
        if needs_const
            && owners
                .consts
                .as_deref()
                .and_then(|c| c.declarations[target.0 as usize])
                .is_none()
        {
            return Err(error(
                package,
                key,
                "public const declaration has missing const library stability attribute",
            ));
        }
        Ok(())
    })?;
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
