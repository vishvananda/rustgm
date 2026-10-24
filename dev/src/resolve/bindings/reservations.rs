//! Completed configured owners reserve source identities before semantic queries.
use super::{
    SourceCore,
    policy::Feature,
    role_attributes::{HeaderAttribute, HeaderAttributes},
};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{
        model::{DefId, DefKind, Resolution, ScopeId},
        package::{Error, Package},
        unit::{FileId, NodeKey, Unit},
    },
    support::{
        bindings::{LangItem, SourceIntrinsic},
        crates::CrateIdentity,
    },
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub definition: DefId,
    pub parent: Option<DefId>,
    pub origin: NodeKey,
}
#[derive(Clone, Copy)]
pub struct IntrinsicDeclaration {
    pub declaration: Declaration,
    pub kind: SourceIntrinsic,
}
pub struct Reservations {
    pub provider: CrateIdentity,
    pub root: ScopeId,
    languages: [Option<Declaration>; LangItem::ALL.len()],
    intrinsics: HashMap<DefId, IntrinsicDeclaration>,
    denied_implementations: HashSet<DefId>,
    pub(super) header_attributes: [HeaderAttributes; LangItem::ALL.len()],
    pub(super) trivial_field_reads: Option<DefId>,
    pub(super) diagnostics: [Option<Declaration>; 3],
    pub steps: usize,
}
impl Reservations {
    #[must_use]
    pub fn language(&self, kind: LangItem) -> Option<Declaration> {
        self.languages[kind.index()]
    }
    /// Reserved identity classifies argument syntax, never implementation evidence.
    #[must_use]
    pub fn callable(&self, definition: DefId) -> Option<crate::support::call_traits::CallTrait> {
        use crate::support::call_traits::CallTrait;
        [
            (LangItem::Fn, CallTrait::Fn),
            (LangItem::FnMut, CallTrait::FnMut),
            (LangItem::FnOnce, CallTrait::FnOnce),
        ]
        .into_iter()
        .find_map(|(role, kind)| {
            self.language(role)
                .filter(|d| d.definition == definition)
                .map(|_| kind)
        })
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
    pub(crate) fn role_attributes(&self, role: LangItem) -> HeaderAttributes {
        self.header_attributes[role.index()]
    }
}
#[derive(Clone, Copy)]
pub(super) enum Claim {
    Language(LangItem),
    Intrinsic,
    DenyImplementations,
    BuiltinMacro,
    Header(HeaderAttribute),
    TrivialFieldReads,
    DiagnosticItem(super::annotations::DiagnosticItem),
}

pub(in crate::resolve) fn finish(
    package: &mut Package,
    facts: &mut Resolution,
) -> Result<(), Error> {
    let mut library = None;
    let claims = claims(package, &mut library)?;
    let inventory = crate::resolve::stability::complete(package, facts, library)?;
    let reserved = reserve_all(package, facts, &claims)?;
    // Both drafts are complete before either owner publishes new state.
    facts.source_bindings = reserved;
    if let (Some(library), Some(inventory)) = (package.library.as_deref_mut(), inventory) {
        library.publish(inventory);
    }
    Ok(())
}

fn reserve_all(
    package: &Package,
    facts: &Resolution,
    claims: &HashMap<NodeKey, Vec<Claim>>,
) -> Result<Option<Box<Reservations>>, Error> {
    let Some(source) = package.source_core() else {
        if !claims.is_empty() {
            return Err(Error::at(
                FileId(0),
                0,
                "compiler bindings require an explicit source-core provider",
            ));
        }
        return Ok(None);
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
        languages: [None; LangItem::ALL.len()],
        intrinsics: HashMap::default(),
        denied_implementations: HashSet::default(),
        header_attributes: [HeaderAttributes::EMPTY; LangItem::ALL.len()],
        trivial_field_reads: None,
        diagnostics: [None; 3],
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
        super::role_attributes::validate(package, source, &mut reserved, declaration, claims)?;
        super::annotations::validate(package, source, &mut reserved, declaration, claims)?;
    }
    if claims.keys().any(|key| !claimed.contains(key)) {
        return Err(Error::at(
            FileId(0),
            0,
            "source binding requires an indexed declaration owner",
        ));
    }
    Ok(Some(Box::new(reserved)))
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
        // Builtin claims were checked by their facade; the remaining complete
        // claims are validated below independently of attribute order.
        Claim::BuiltinMacro
        | Claim::Header(_)
        | Claim::TrivialFieldReads
        | Claim::DiagnosticItem(_) => {}
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
            let kind = SourceIntrinsic::from_name(spelling)
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

fn claims(
    package: &Package,
    library: &mut Option<crate::resolve::stability::Draft>,
) -> Result<HashMap<NodeKey, Vec<Claim>>, Error> {
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
            let header = super::panic_contracts::checked(
                package,
                NodeKey {
                    file,
                    node: owner.node,
                },
            )?;
            let contract = header
                .as_ref()
                .map(super::panic_contracts::CheckedHeader::summary);
            if contract.and_then(|contract| contract.builtin).is_some() {
                result
                    .entry(NodeKey { file, node })
                    .or_default()
                    .push(Claim::BuiltinMacro);
            }
            let mut transparency = false;
            for (index, attribute, forest) in
                unit.retained_indexed_attributes(owner.attributes.clone())
            {
                if crate::resolve::index::unavailable_macros::collect_transparency(
                    package,
                    unit,
                    NodeKey { file, node },
                    attribute,
                    forest,
                    &mut transparency,
                )? {
                    continue;
                }
                let allowance = header
                    .as_ref()
                    .map(|header| header.allowance_at(index, attribute))
                    .transpose()?
                    .flatten();
                if crate::resolve::stability::collect(
                    package.library.as_deref(),
                    library,
                    unit,
                    NodeKey { file, node },
                    attribute,
                    forest,
                    allowance,
                )? {
                    continue;
                }
                if let Some(claim) = attribute_claim(unit, file, attribute, forest)? {
                    result
                        .entry(NodeKey { file, node })
                        .or_default()
                        .push(claim);
                }
            }
        }
    }
    Ok(result)
}

/// Decode one declaration claim after its complete macro/stability header was checked.
fn attribute_claim(
    unit: &Unit,
    file: FileId,
    attribute: Attribute,
    forest: &Forest,
) -> Result<Option<Claim>, Error> {
    let error = |message| Error::at(file, attribute.span.start, message);
    let name = attribute.name(forest, unit.source.text());
    if let Some(claim) = name
        .map(|name| super::role_attributes::claim(name, attribute.input))
        .transpose()
        .map_err(error)?
        .flatten()
    {
        return Ok(Some(claim));
    }
    let claim = match name {
        Some("lang") => language_claim(unit, file, attribute, forest)?,
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
        Some("rustc_diagnostic_item") => {
            Claim::DiagnosticItem(super::annotations::parse(unit, file, attribute, forest)?)
        }
        _ => return Ok(None),
    };
    Ok(Some(claim))
}

fn language_claim(
    unit: &Unit,
    file: FileId,
    attribute: Attribute,
    forest: &Forest,
) -> Result<Claim, Error> {
    let error = |message| Error::at(file, attribute.span.start, message);
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
    let name =
        std::str::from_utf8(&bytes).map_err(|_| error("language item name requires UTF-8"))?;
    Ok(Claim::Language(LangItem::from_name(name).ok_or_else(
        || error("unsupported source language item"),
    )?))
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
