//! Complete canonical membership and exact source lexemes share cold buffers.
use super::{
    Declaration, Entry, HeaderData, KnownFeature, Owner, Package, State, Witness, grants, is_macro,
    require_features, witness::Capture,
};
use crate::{
    resolve::unit::editions,
    support::hash::HashSet,
    syntax::{
        ast::Kind,
        attribute::{self, Attribute, Input},
        token_tree::{Delimiter, Forest, TreeKind},
    },
};
use std::{borrow::Cow, rc::Rc};
use unicode_normalization::UnicodeNormalization;

#[derive(Default)]
pub(super) struct Builder {
    pool: HashSet<Rc<str>>,
    features: HashSet<Rc<str>>,
    pool_bytes: usize,
}
impl Builder {
    pub(super) fn parse(
        &mut self,
        package: &Package,
        owner: &Owner<'_>,
        forest: &Forest,
        range: std::ops::Range<u32>,
        work: &mut State,
    ) -> Result<(), &'static str> {
        work.charge((range.end - range.start) as usize, 0)?;
        let mut index = range.start;
        while index < range.end {
            let token = forest
                .token(index)
                .ok_or("invalid internal macro feature")?;
            let edition = editions::token(owner.node.file, token, |file| {
                package.units.get(file.0 as usize)
            })
            .map_err(|error| error.message)?;
            if !attribute::word(forest, owner.unit.source.text(), index, edition)
                .map_err(|error| error.message)?
            {
                return Err("internal macro allowance requires identifier names");
            }
            let name = forest
                .name(owner.unit.source.text(), index)
                .ok_or("invalid internal macro feature")?;
            self.feature(name, work)?;
            index += 1;
            if index == range.end {
                break;
            }
            if !forest.punctuation(index, crate::lex::token::Punctuation::Comma) {
                return Err("internal macro allowance list is not implemented");
            }
            index += 1;
        }
        Ok(())
    }
    fn feature(&mut self, name: &str, work: &mut State) -> Result<(), &'static str> {
        work.charge(name.len().saturating_mul(4), 0)?;
        let canonical = if name.is_ascii() {
            Cow::Borrowed(name)
        } else {
            // NFC uses the same pinned implementation as the ordinary name arena.
            work.charge(0, name.len().saturating_mul(8))?;
            let canonical = name.nfc().collect::<String>();
            work.charge(canonical.len(), canonical.capacity())?;
            Cow::Owned(canonical)
        };
        let symbol = self.lexeme(&canonical, work)?;
        work.charge(canonical.len(), 0)?;
        if self.features.contains(canonical.as_ref()) {
            return Ok(());
        }
        work.charge(1, 4 * size_of::<Rc<str>>())?;
        self.features
            .try_reserve(1)
            .map_err(|_| "macro feature allocation failed")?;
        self.features.insert(symbol);
        self.check_storage(work)
    }
    pub(super) fn lexeme(&mut self, name: &str, work: &mut State) -> Result<Rc<str>, &'static str> {
        work.charge(name.len(), 0)?;
        if let Some(existing) = self.pool.get(name) {
            return Ok(Rc::clone(existing));
        }
        work.charge(1, name.len().saturating_add(4 * size_of::<Rc<str>>()))?;
        self.pool
            .try_reserve(1)
            .map_err(|_| "macro lexeme allocation failed")?;
        let spelling: Rc<str> = name.into();
        self.pool.insert(Rc::clone(&spelling));
        self.pool_bytes = self.pool_bytes.saturating_add(name.len());
        self.check_storage(work)?;
        Ok(spelling)
    }
    fn check_storage(&self, work: &mut State) -> Result<(), &'static str> {
        let bytes = self
            .pool
            .capacity()
            .saturating_add(self.features.capacity())
            .saturating_mul(size_of::<Rc<str>>())
            .saturating_add(self.pool_bytes);
        work.payload(bytes)
    }
    pub(super) fn finish(self) -> (HashSet<Rc<str>>, usize) {
        (self.features, self.pool_bytes)
    }
}

pub(super) fn collect(
    package: &Package,
    owner: &Owner<'_>,
    work: &mut State,
) -> Result<Option<Entry>, crate::resolve::package::Error> {
    let error = |message| {
        crate::resolve::package::Error::at(
            owner.key.file,
            owner.unit.configured.ast.node(owner.node.node).span.start,
            message,
        )
    };
    let mut names = Builder::default();
    let mut declaration = read(package, owner, &mut names, work)?;
    if declaration.builtin.is_none()
        && !declaration.has_allowance
        && !declaration.unsafe_lint
        && (package.source_core().is_none() || !is_macro(owner.unit, owner.node))
    {
        return Ok(None);
    }
    let span = owner.unit.configured.ast.node(owner.node.node).span;
    declaration.edition = editions::introducer(owner.node.file, span, |file| {
        package.units.get(file.0 as usize)
    })
    .map_err(|failure| error(failure.message))?;
    if declaration.builtin.is_none() && !declaration.has_allowance && !declaration.unsafe_lint {
        return Ok(Some(Entry::Plain(declaration)));
    }
    let witnesses = capture(owner, declaration.builtin, &mut names, work).map_err(error)?;
    let (features, lexeme_bytes) = names.finish();
    for feature in [
        KnownFeature::EditionPanic,
        KnownFeature::PanicInternals,
        KnownFeature::CoreIntrinsics,
        KnownFeature::CfgUbChecks,
    ] {
        work.charge(feature.name().spelling().len(), 0)
            .map_err(error)?;
        if features.contains(feature.name().spelling()) {
            declaration.consumers |= feature.mask();
        }
    }
    let witness_work = 1 + witnesses
        .iter()
        .map(|witness| {
            1 + witness
                .nodes
                .iter()
                .map(|(_, _, spelling)| {
                    1 + spelling
                        .as_ref()
                        .map_or(0, |spelling| spelling.name().len())
                })
                .sum::<usize>()
        })
        .sum::<usize>();
    grants::retain(
        HeaderData {
            declaration,
            witnesses,
            features,
            witness_work,
            lexeme_bytes,
        },
        work,
    )
    .map(Some)
    .map_err(error)
}

fn read(
    package: &Package,
    owner: &Owner<'_>,
    names: &mut Builder,
    work: &mut State,
) -> Result<Declaration, crate::resolve::package::Error> {
    let mut declaration = Declaration {
        node: owner.node,
        owner: owner.key,
        attributes: (owner.attributes.start, owner.attributes.end),
        builtin: None,
        has_allowance: false,
        unsafe_lint: false,
        consumers: 0,
        edition: owner.unit.edition(),
    };
    for index in owner.attributes.clone() {
        let effect = &owner.unit.configured.effects[index];
        if effect.attribute.effect != crate::expand::configured::Effect::Keep {
            continue;
        }
        let attribute = effect.attribute.attribute;
        let forest = &owner.unit.configured.ast.attributes[effect.capture as usize].forest;
        let error = |message| {
            crate::resolve::package::Error::at(owner.key.file, attribute.span.start, message)
        };
        work.charge(1, 0).map_err(error)?;
        match attribute.name(forest, owner.unit.source.text()) {
            Some("rustc_builtin_macro") => {
                if declaration.builtin.is_some() {
                    return Err(error("duplicate source builtin macro claim"));
                }
                declaration.builtin = Some(super::super::source_builtin::parse(
                    package, owner.node, attribute, forest,
                )?);
            }
            Some("allow_internal_unsafe") => {
                declaration.unsafe_lint = true;
                require_features(package, declaration).map_err(error)?;
                unsafe_lint_attribute(owner, attribute).map_err(error)?;
            }
            Some("allow_internal_unstable") => {
                declaration.has_allowance = true;
                require_features(package, declaration).map_err(error)?;
                let range = allowance_range(owner, attribute, forest).map_err(error)?;
                names
                    .parse(package, owner, forest, range, work)
                    .map_err(error)?;
            }
            _ => {}
        }
    }
    Ok(declaration)
}

fn capture(
    owner: &Owner<'_>,
    builtin: Option<crate::expand::builtin::Builtin>,
    names: &mut Builder,
    work: &mut State,
) -> Result<Vec<Witness>, &'static str> {
    let mut witnesses = Vec::new();
    for index in owner.attributes.clone() {
        work.charge(1, 0)?;
        let effect = &owner.unit.configured.effects[index];
        if effect.attribute.effect != crate::expand::configured::Effect::Keep {
            continue;
        }
        let attribute = effect.attribute.attribute;
        let forest = &owner.unit.configured.ast.attributes[effect.capture as usize].forest;
        let name = attribute.name(forest, owner.unit.source.text());
        if !matches!(
            name,
            Some("rustc_builtin_macro" | "allow_internal_unstable" | "allow_internal_unsafe")
        ) {
            continue;
        }
        work.charge(1, 4 * size_of::<Witness>())?;
        witnesses
            .try_reserve(1)
            .map_err(|_| "macro witness allocation failed")?;
        witnesses.push(Witness::capture(
            Capture {
                owner,
                index,
                attribute,
                forest,
                builtin,
                allowance: matches!(
                    name,
                    Some("allow_internal_unstable" | "allow_internal_unsafe")
                ),
            },
            names,
            work,
        )?);
    }
    Ok(witnesses)
}

fn allowance_range(
    owner: &Owner<'_>,
    attribute: Attribute,
    forest: &Forest,
) -> Result<std::ops::Range<u32>, &'static str> {
    if attribute.unsafe_wrapper {
        return Err("internal macro allowance cannot use an unsafe wrapper");
    }
    if !matches!(
        owner.unit.configured.ast.node(owner.node.node).kind,
        Kind::Item(_)
    ) || !is_macro(owner.unit, owner.node)
    {
        return Err("internal macro allowance requires a macro declaration");
    }
    let Input::Delimited(group) = attribute.input else {
        return Err("invalid internal macro allowance arguments");
    };
    if !forest.tree(group).is_some_and(|tree| {
        matches!(
            tree.kind,
            TreeKind::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            }
        )
    }) {
        return Err("invalid internal macro allowance arguments");
    }
    forest
        .children(group)
        .map(|range| range.start..range.end)
        .ok_or("invalid internal macro allowance arguments")
}

fn unsafe_lint_attribute(owner: &Owner<'_>, attribute: Attribute) -> Result<(), &'static str> {
    let Kind::Item(index) = owner.unit.configured.ast.node(owner.node.node).kind else {
        return Err("internal unsafe lint metadata requires an actual macro or function");
    };
    if !matches!(
        owner.unit.configured.ast.items[index as usize].kind,
        crate::syntax::item::ItemKind::MacroRules { .. }
            | crate::syntax::item::ItemKind::DeclarativeMacro { .. }
            | crate::syntax::item::ItemKind::Function { .. }
    ) {
        return Err("internal unsafe lint metadata requires an actual macro or function");
    }
    if attribute.input != Input::Empty || attribute.unsafe_wrapper {
        return Err("allow_internal_unsafe requires a plain no-arguments attribute");
    }
    Ok(())
}
