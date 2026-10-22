//! Completed configured owners reserve source identities before semantic queries.
use super::{SourceCore, policy::Feature};
use crate::{
    resolve::{
        model::{DefId, DefKind, Resolution, ScopeId},
        package::{Error, Package},
        unit::{FileId, NodeKey},
    },
    support::{
        bindings::{LangItem, LayoutIntrinsic},
        crates::CrateIdentity,
    },
    syntax::{ast::Kind, attribute::Input, item::ItemKind},
};
use crate::support::hash::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub definition: DefId,
    pub parent: Option<DefId>,
    pub origin: NodeKey,
}
#[derive(Clone, Copy)]
pub struct IntrinsicDeclaration {
    pub declaration: Declaration,
    pub kind: LayoutIntrinsic,
}
pub struct Reservations {
    pub provider: CrateIdentity,
    pub root: ScopeId,
    languages: [Option<Declaration>; 5],
    intrinsics: HashMap<DefId, IntrinsicDeclaration>,
    denied_implementations: HashSet<DefId>,
    pub steps: usize,
}
impl Reservations {
    #[must_use]
    pub fn language(&self, kind: LangItem) -> Option<Declaration> {
        self.languages[kind.index()]
    }
    pub fn intrinsics(&self) -> impl Iterator<Item = IntrinsicDeclaration> + '_ {
        self.intrinsics.values().copied()
    }
    #[must_use]
    pub fn intrinsic(&self, definition: DefId) -> Option<IntrinsicDeclaration> {
        self.intrinsics.get(&definition).copied()
    }
    #[must_use]
    pub fn denies_implementation(&self, definition: DefId) -> bool {
        self.denied_implementations.contains(&definition)
    }
}
#[derive(Clone, Copy)]
enum Claim {
    Language(LangItem),
    Intrinsic,
    DenyImplementations,
    BuiltinMacro,
}

pub(in crate::resolve) fn finish(package: &Package, facts: &mut Resolution) -> Result<(), Error> {
    let claims = claims(package)?;
    let Some(source) = package.source_core.as_deref() else {
        if !claims.is_empty() {
            return Err(Error::at(
                FileId(0),
                0,
                "compiler bindings require an explicit source-core provider",
            ));
        }
        return Ok(());
    };
    if facts.source_bindings.is_some() {
        return Err(Error::at(
            FileId(0),
            0,
            "source bindings were already reserved",
        ));
    }
    if !facts.builtin_traits.bootstrap_absent()
        || crate::support::library_types::LibraryType::ALL
            .into_iter()
            .any(|kind| facts.library_types.get(kind).is_some())
        || facts.intrinsics.iter().next().is_some()
        || facts.drop_trait.is_some()
        || !facts.macro_prelude.is_empty()
    {
        return Err(Error::at(
            FileId(0),
            0,
            "source bindings cannot share bootstrap identities",
        ));
    }
    let mut reserved = Reservations {
        provider: source.provider.clone(),
        root: *facts
            .module_scopes
            .first()
            .ok_or_else(|| Error::at(FileId(0), 0, "missing source crate root"))?,
        languages: [None; 5],
        intrinsics: HashMap::default(),
        denied_implementations: HashSet::default(),
        steps: 0,
    };
    let parents = parents(package, facts);
    let mut claimed = HashSet::default();
    for (index, definition) in facts.definitions.iter().enumerate() {
        reserved.steps += 1;
        let Some(key) = definition.node else {
            continue;
        };
        if !primary(package, key, definition.kind) {
            continue;
        }
        let id = DefId(
            u32::try_from(index)
                .map_err(|_| Error::at(key.file, 0, "source definition space exhausted"))?,
        );
        prototype(package, facts, key, id, claims.get(&key))?;
        let Some(claims) = claims.get(&key) else {
            continue;
        };
        claimed.insert(key);
        let declaration = Declaration {
            definition: id,
            parent: parents.get(&definition.scope).copied(),
            origin: key,
        };
        for claim in claims {
            reserved.steps += 1;
            reserve(package, source, &mut reserved, declaration, *claim)?;
        }
    }
    if claims.keys().any(|key| !claimed.contains(key)) {
        return Err(Error::at(
            FileId(0),
            0,
            "source binding requires an indexed declaration owner",
        ));
    }
    facts.source_bindings = Some(Box::new(reserved));
    Ok(())
}

fn reserve(
    package: &Package,
    source: &SourceCore,
    reserved: &mut Reservations,
    declaration: Declaration,
    claim: Claim,
) -> Result<(), Error> {
    let error = |message| {
        Error::at(
            declaration.origin.file,
            package
                .unit(declaration.origin.file)
                .configured
                .ast
                .node(declaration.origin.node)
                .span
                .start,
            message,
        )
    };
    match claim {
        // The configured-owner facade has already validated this exact declaration.
        Claim::BuiltinMacro => {}
        Claim::DenyImplementations => {
            if !source.features.has(Feature::RustcAttributes) {
                return Err(error("implementation restriction requires rustc_attrs"));
            }
            let unit = package.unit(declaration.origin.file);
            let Kind::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
                return Err(error("implementation restriction requires a trait owner"));
            };
            if !matches!(
                unit.configured.ast.items[index as usize].kind,
                ItemKind::Trait { .. }
            ) {
                return Err(error(
                    "implementation restriction requires a trait declaration",
                ));
            }
            reserved
                .denied_implementations
                .insert(declaration.definition);
        }
        Claim::Language(kind) => {
            if !source.features.has(Feature::LangItems) {
                return Err(error("lang requires its explicit source feature"));
            }
            if reserved.languages[kind.index()]
                .replace(declaration)
                .is_some()
            {
                return Err(error("duplicate source language item"));
            }
        }
        Claim::Intrinsic => {
            if !source.features.has(Feature::Intrinsics)
                || !source.features.has(Feature::RustcAttributes)
            {
                return Err(error("intrinsic binding requires explicit source features"));
            }
            let unit = package.unit(declaration.origin.file);
            let Kind::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
                return Err(error("intrinsic binding has no item owner"));
            };
            let item = unit.configured.ast.items[index as usize];
            if !matches!(item.kind, ItemKind::Function { .. }) {
                return Err(error("intrinsic binding requires a function"));
            }
            if declaration.parent.is_some() {
                return Err(error("intrinsic binding requires a module function"));
            }
            let span = item
                .name
                .ok_or_else(|| error("intrinsic function has no name"))?;
            let spelling = unit
                .spelling(span)
                .map_err(|e| Error::at(declaration.origin.file, e.offset, e.message))?;
            let kind = LayoutIntrinsic::from_name(spelling)
                .ok_or_else(|| error("unsupported source intrinsic"))?;
            if reserved
                .intrinsics
                .insert(
                    declaration.definition,
                    IntrinsicDeclaration { declaration, kind },
                )
                .is_some()
            {
                return Err(error("duplicate intrinsic binding on one declaration"));
            }
        }
    }
    Ok(())
}

fn prototype(
    package: &Package,
    facts: &Resolution,
    key: NodeKey,
    definition: DefId,
    claims: Option<&Vec<Claim>>,
) -> Result<(), Error> {
    let unit = package.unit(key.file);
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Ok(());
    };
    if matches!(
        unit.configured.ast.items[index as usize].kind,
        ItemKind::Function { body: None, .. }
    ) && !facts.foreign_owners.contains_key(&definition)
        && facts.scopes[facts.definitions[definition.0 as usize].scope.0 as usize]
            .trait_access
            .is_none()
        && !claims
            .is_some_and(|claims| claims.iter().any(|claim| matches!(claim, Claim::Intrinsic)))
    {
        return Err(Error::at(
            key.file,
            unit.configured.ast.node(key.node).span.start,
            "function requires a definition or an enabled source intrinsic binding",
        ));
    }
    Ok(())
}

fn primary(package: &Package, key: NodeKey, kind: DefKind) -> bool {
    let unit = package.unit(key.file);
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return false;
    };
    matches!(
        (unit.configured.ast.items[index as usize].kind, kind),
        (ItemKind::Trait { .. }, DefKind::Trait)
            | (ItemKind::Function { .. }, DefKind::Function)
            | (ItemKind::Const { .. }, DefKind::Const)
            | (ItemKind::Static { .. }, DefKind::Static)
            | (ItemKind::Struct { .. }, DefKind::Struct)
            | (ItemKind::TypeAlias { .. }, DefKind::TypeAlias)
            | (ItemKind::Impl { .. }, DefKind::SelfType)
            | (ItemKind::MacroRules { .. }, DefKind::Macro)
    )
}
fn parents(package: &Package, facts: &Resolution) -> HashMap<ScopeId, DefId> {
    facts
        .definitions
        .iter()
        .enumerate()
        .filter_map(|(index, definition)| {
            let key = definition.node?;
            if !primary(package, key, definition.kind)
                || !matches!(definition.kind, DefKind::Trait | DefKind::SelfType)
            {
                return None;
            }
            Some((definition.members?, DefId(u32::try_from(index).ok()?)))
        })
        .collect()
}

fn claims(package: &Package) -> Result<HashMap<NodeKey, Vec<Claim>>, Error> {
    let mut result: HashMap<NodeKey, Vec<Claim>> = HashMap::default();
    for (file, unit) in package.units.iter().enumerate() {
        let file = FileId(
            u32::try_from(file)
                .map_err(|_| Error::at(FileId(0), 0, "source file space exhausted"))?,
        );
        for owner in unit.configured.owners.iter().filter(|o| o.enabled) {
            let mut node = owner.node;
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                node = value;
            }
            if super::source_builtin::claim(
                package,
                NodeKey {
                    file,
                    node: owner.node,
                },
            )?
            .is_some()
            {
                result
                    .entry(NodeKey { file, node })
                    .or_default()
                    .push(Claim::BuiltinMacro);
            }
            for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
                let error = |message| Error::at(file, attribute.span.start, message);
                let claim = match attribute.name(forest, unit.source.text()) {
                    Some("lang") => {
                        let Input::Equals(range) = attribute.input else {
                            return Err(error("lang requires a string literal"));
                        };
                        if range.end != range.start + 1 {
                            return Err(error("lang requires one string literal"));
                        }
                        let token = forest
                            .token(range.start)
                            .ok_or_else(|| error("lang requires a string literal"))?;
                        let Some(crate::literal::Value::String(bytes)) =
                            crate::literal::interpret(unit.source.text(), token)
                                .map_err(|e| Error::at(file, e.offset, e.message))?
                        else {
                            return Err(error("lang requires a string literal"));
                        };
                        let name = std::str::from_utf8(&bytes)
                            .map_err(|_| error("language item name requires UTF-8"))?;
                        Claim::Language(
                            LangItem::from_name(name)
                                .ok_or_else(|| error("unsupported source language item"))?,
                        )
                    }
                    Some("rustc_intrinsic") => {
                        if attribute.input != Input::Empty {
                            return Err(error("rustc_intrinsic requires no arguments"));
                        }
                        Claim::Intrinsic
                    }
                    Some("rustc_deny_explicit_impl") => {
                        if attribute.input != Input::Empty {
                            return Err(error("rustc_deny_explicit_impl requires no arguments"));
                        }
                        Claim::DenyImplementations
                    }
                    _ => continue,
                };
                result
                    .entry(NodeKey { file, node })
                    .or_default()
                    .push(claim);
            }
        }
    }
    Ok(result)
}
