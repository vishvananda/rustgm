//! Source-owned macro claims and the one implemented internal edition permission.
use super::policy::Feature;
mod witness;
use crate::{
    expand::{builtin::Builtin, marks::ExpansionId},
    lex::token::Punctuation,
    resolve::{
        package::{Error, ModuleId, Package},
        unit::{NodeKey, Unit},
    },
    support::{edition::Edition, hash::HashMap},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::{Delimiter, Forest, Span, TreeKind},
    },
};
use std::cell::RefCell;
use witness::{Node as WitnessNode, Witness};

#[derive(Clone, Copy)]
pub(in crate::resolve) struct Allowance {
    declaration: NodeKey,
    attribute: Attribute,
}
impl Allowance {
    pub(in crate::resolve) fn matches(self, key: NodeKey, attribute: Attribute) -> bool {
        self.declaration == key && self.attribute == attribute
    }
}
#[derive(Clone, Copy)]
pub(in crate::resolve) struct Declaration {
    pub node: NodeKey,
    owner: NodeKey,
    attributes: (usize, usize),
    pub builtin: Option<Builtin>,
    pub allowance: Option<Allowance>,
    pub edition: Edition,
}
#[derive(Default)]
struct State {
    declarations: HashMap<NodeKey, Entry>,
    editions: HashMap<ExpansionId, (Frame, Edition)>,
    steps: usize,
    bytes: usize,
    failed: bool,
    witness_bytes: usize,
}
impl State {
    fn charge(&mut self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(steps);
        self.bytes = self.bytes.saturating_add(bytes);
        self.failed |= self.steps > 1_000_000 || self.bytes > 64 * 1024 * 1024;
        if self.failed {
            return Err("source macro contracts exceed their work or storage budget");
        }
        Ok(())
    }
    /// Payload accounting is additional to conservative attempted allocation charges.
    fn payload(&mut self, extra: usize) -> Result<(), &'static str> {
        let bytes = self
            .declarations
            .capacity()
            .saturating_mul(size_of::<(NodeKey, Entry)>())
            .saturating_add(
                self.editions
                    .capacity()
                    .saturating_mul(size_of::<(ExpansionId, (Frame, Edition))>()),
            )
            .saturating_add(self.witness_bytes)
            .saturating_add(extra);
        self.failed |= bytes > 64 * 1024 * 1024;
        self.charge(0, 0)
    }
}
#[derive(Default)]
pub(in crate::resolve) struct Contracts(RefCell<State>);

/// A cold cache authenticates its actual frame, never only an arena integer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::resolve) struct Frame {
    pub context: ExpansionId,
    pub definition: usize,
    pub call: NodeKey,
    pub original: Span,
    pub parent: ExpansionId,
    pub owner: Option<NodeKey>,
    pub module: ModuleId,
    pub edition: Edition,
    pub skip: bool,
}

struct Owner<'a> {
    unit: &'a Unit,
    key: NodeKey,
    node: NodeKey,
    attributes: std::ops::Range<usize>,
}
struct Entry {
    declaration: Declaration,
    witnesses: Vec<Witness>,
    witness_work: usize,
}
fn owner(package: &Package, key: NodeKey) -> Result<Option<Owner<'_>>, Error> {
    let unit = package
        .units
        .get(key.file.0 as usize)
        .ok_or_else(|| Error::at(key.file, 0, "unknown source macro file"))?;
    if key.node.get() as usize >= unit.configured.ast.nodes.len() {
        return Err(Error::at(key.file, 0, "unknown source macro node"));
    }
    let Some(index) = unit.owner_indices[key.node.get() as usize] else {
        return Ok(
            (unit.configured.active[key.node.get() as usize] && is_macro(unit, key)).then_some(
                Owner {
                    unit,
                    key,
                    node: key,
                    attributes: 0..0,
                },
            ),
        );
    };
    let owner = &unit.configured.owners[index.get() as usize - 1];
    if !owner.enabled {
        return Ok(None);
    }
    let mut node = owner.node;
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
        node = value;
    }
    Ok(Some(Owner {
        unit,
        key: NodeKey {
            node: owner.node,
            ..key
        },
        node: NodeKey { node, ..key },
        attributes: owner.attributes.clone(),
    }))
}
pub(in crate::resolve) fn declaration(
    package: &Package,
    key: NodeKey,
) -> Result<Option<Declaration>, Error> {
    let Some(owner) = owner(package, key)? else {
        return Ok(None);
    };
    let error = |message| {
        Error::at(
            key.file,
            owner.unit.configured.ast.node(owner.node.node).span.start,
            message,
        )
    };
    if let Some(source) = package.source_core() {
        if source.current != source.provider {
            return Err(error("source macro provider identity changed"));
        }
        let mut state = source
            .macros
            .0
            .try_borrow_mut()
            .map_err(|_| error("reentrant source macro contract query"))?;
        state.charge(1, 0).map_err(error)?;
        if let Some((entry, cost)) = state
            .declarations
            .get(&owner.node)
            .map(|entry| (entry.declaration, entry.witness_work))
        {
            state.charge(cost, 0).map_err(error)?;
            if entry.owner != owner.key
                || entry.attributes != (owner.attributes.start, owner.attributes.end)
                || !is_macro(owner.unit, owner.node)
                || state.declarations.get(&owner.node).is_none_or(|entry| {
                    !entry
                        .witnesses
                        .iter()
                        .all(|witness| witness.valid(owner.unit))
                })
            {
                return Err(error("source macro attribute owner changed"));
            }
            require_features(package, entry).map_err(error)?;
            return Ok(Some(entry));
        }
        let entry = parse(package, &owner, &mut state)?;
        let result = entry.as_ref().map(|entry| entry.declaration);
        if let Some(entry) = entry {
            state
                .charge(1, 4 * size_of::<(NodeKey, Entry)>())
                .map_err(error)?;
            state
                .declarations
                .try_reserve(1)
                .map_err(|_| error("source macro contract allocation failed"))?;
            let bytes = entry
                .witnesses
                .capacity()
                .saturating_mul(size_of::<Witness>())
                .saturating_add(
                    entry
                        .witnesses
                        .iter()
                        .map(|witness| {
                            witness
                                .nodes
                                .capacity()
                                .saturating_mul(size_of::<WitnessNode>())
                        })
                        .sum::<usize>(),
                );
            state.payload(bytes).map_err(error)?;
            state.witness_bytes += bytes;
            state.declarations.insert(entry.declaration.node, entry);
        }
        Ok(result)
    } else {
        Ok(parse(package, &owner, &mut State::default())?.map(|entry| entry.declaration))
    }
}
pub(in crate::resolve) fn existing(
    package: &Package,
    key: NodeKey,
) -> Result<Option<Declaration>, Error> {
    let source = package
        .source_core()
        .ok_or_else(|| Error::at(key.file, 0, "missing source macro provider"))?;
    let owner = {
        let mut state = source
            .macros
            .0
            .try_borrow_mut()
            .map_err(|_| Error::at(key.file, 0, "reentrant source macro contract query"))?;
        state
            .charge(1, 0)
            .map_err(|message| Error::at(key.file, 0, message))?;
        state
            .declarations
            .get(&key)
            .map(|entry| entry.declaration.owner)
    };
    owner
        .map(|owner| declaration(package, owner))
        .transpose()
        .map(Option::flatten)
}
fn require_features(package: &Package, entry: Declaration) -> Result<(), &'static str> {
    let source = package
        .source_core()
        .ok_or("internal macro contracts require an actual source provider")?;
    if entry.builtin.is_some() && !source.features.has(Feature::RustcAttributes) {
        return Err("builtin macro binding requires rustc_attrs");
    }
    if entry.allowance.is_some() && !source.features.has(Feature::InternalMacroAllowance) {
        return Err("internal macro allowance requires allow_internal_unstable");
    }
    Ok(())
}
fn parse(package: &Package, owner: &Owner<'_>, work: &mut State) -> Result<Option<Entry>, Error> {
    let mut result = Declaration {
        node: owner.node,
        owner: owner.key,
        attributes: (owner.attributes.start, owner.attributes.end),
        builtin: None,
        allowance: None,
        edition: owner.unit.edition(),
    };
    let mut witnesses = Vec::new();
    let mut witness_work = 0;
    for index in owner.attributes.clone() {
        let effect = &owner.unit.configured.effects[index];
        if effect.attribute.effect != crate::expand::configured::Effect::Keep {
            continue;
        }
        let attribute = effect.attribute.attribute;
        let forest = &owner.unit.configured.ast.attributes[effect.capture as usize].forest;
        let error = |message| Error::at(owner.key.file, attribute.span.start, message);
        work.charge(1, 0).map_err(error)?;
        match attribute.name(forest, owner.unit.source.text()) {
            Some("rustc_builtin_macro") => {
                if result.builtin.is_some() {
                    return Err(error("duplicate source builtin macro claim"));
                }
                result.builtin = Some(super::source_builtin::parse(
                    package, owner.node, attribute, forest,
                )?);
            }
            Some("allow_internal_unstable") => {
                if result.allowance.is_some() {
                    return Err(error(
                        "repeated internal macro allowance is not implemented",
                    ));
                }
                allowance(owner, attribute, forest).map_err(error)?;
                result.allowance = Some(Allowance {
                    declaration: owner.node,
                    attribute,
                });
                require_features(package, result).map_err(error)?;
            }
            _ => continue,
        }
        let witness = Witness::capture(owner, index, attribute, forest, result.builtin, work)
            .map_err(error)?;
        work.charge(1, 4 * size_of::<Witness>()).map_err(error)?;
        witness_work += 1 + witness
            .nodes
            .iter()
            .map(|(_, _, spelling)| 1 + spelling.map_or(0, |spelling| spelling.name().len()))
            .sum::<usize>();
        witnesses.push(witness);
    }
    if result.builtin.is_some()
        || result.allowance.is_some()
        || (package.source_core().is_some() && is_macro(owner.unit, owner.node))
    {
        let span = owner.unit.configured.ast.node(owner.node.node).span;
        result.edition = crate::resolve::unit::editions::introducer(owner.node.file, span, |id| {
            package.units.get(id.0 as usize)
        })
        .map_err(|error| Error::at(owner.node.file, error.offset, error.message))?;
        return Ok(Some(Entry {
            declaration: result,
            witnesses,
            witness_work,
        }));
    }
    Ok(None)
}
fn is_macro(unit: &Unit, key: NodeKey) -> bool {
    let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
        return false;
    };
    matches!(
        unit.configured.ast.items[index as usize].kind,
        ItemKind::MacroRules { .. } | ItemKind::DeclarativeMacro { .. }
    )
}
fn allowance(owner: &Owner<'_>, attribute: Attribute, forest: &Forest) -> Result<(), &'static str> {
    if attribute.unsafe_wrapper {
        return Err("internal macro allowance cannot use an unsafe wrapper");
    }
    let Kind::Item(index) = owner.unit.configured.ast.node(owner.node.node).kind else {
        return Err("internal macro allowance requires a macro declaration");
    };
    if !matches!(
        owner.unit.configured.ast.items[index as usize].kind,
        ItemKind::MacroRules { .. } | ItemKind::DeclarativeMacro { .. }
    ) {
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
    let range = forest
        .children(group)
        .ok_or("invalid internal macro allowance arguments")?;
    if forest.name(owner.unit.source.text(), range.start) != Some("edition_panic") {
        return Err("internal macro allowance feature is not implemented");
    }
    let end = range.start + 1 + u32::from(forest.punctuation(range.start + 1, Punctuation::Comma));
    if end != range.end {
        return Err("internal macro allowance list is not implemented");
    }
    Ok(())
}
impl Contracts {
    pub(in crate::resolve) fn charge(
        &self,
        steps: usize,
        bytes: usize,
    ) -> Result<(), &'static str> {
        self.0
            .try_borrow_mut()
            .map_err(|_| "reentrant source macro contract query")?
            .charge(steps, bytes)
    }
    pub(in crate::resolve) fn edition(
        &self,
        frame: Frame,
    ) -> Result<Option<Edition>, &'static str> {
        let mut state = self
            .0
            .try_borrow_mut()
            .map_err(|_| "reentrant panic edition query")?;
        state.charge(1, 0)?;
        if let Some(&(actual, edition)) = state.editions.get(&frame.context) {
            if actual != frame {
                return Err("panic edition frame identity changed");
            }
            return Ok(Some(edition));
        }
        Ok(None)
    }
    pub(in crate::resolve) fn retain_editions(
        &self,
        frames: &[Frame],
        edition: Edition,
    ) -> Result<(), &'static str> {
        let mut state = self
            .0
            .try_borrow_mut()
            .map_err(|_| "reentrant panic edition query")?;
        state.charge(
            frames.len(),
            frames
                .len()
                .saturating_mul(4 * size_of::<(ExpansionId, (Frame, Edition))>()),
        )?;
        if frames.iter().any(|frame| {
            frame.context.0 == 0
                || state
                    .editions
                    .get(&frame.context)
                    .is_some_and(|value| *value != (*frame, edition))
        }) {
            return Err("panic edition frame identity changed");
        }
        state
            .editions
            .try_reserve(frames.len())
            .map_err(|_| "panic edition cache allocation failed")?;
        state.payload(0)?;
        for &frame in frames {
            state.editions.insert(frame.context, (frame, edition));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/panic_permissions.rs"]
mod tests;
