//! Source-owned macro claims and the one implemented internal edition permission.
use super::KnownFeature;
mod grants;
mod lists;
mod witness;
use crate::{
    expand::{builtin::Builtin, marks::ExpansionId},
    resolve::{
        package::{Error, ModuleId, Package},
        unit::{NodeKey, Unit},
    },
    support::{edition::Edition, hash::HashMap},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::{Forest, Span},
    },
};
use grants::Entry;
pub(in crate::resolve) use grants::{CheckedHeader, HeaderData};
use std::{cell::RefCell, rc::Rc};
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
    pub has_allowance: bool,
    /// Suppresses only the unsafe-code lint; it grants no unsafe operation.
    pub unsafe_lint: bool,
    consumers: u8,
    pub edition: Edition,
}
impl Declaration {
    /// The configured attributed owner retained by the actual header producer.
    pub(in crate::resolve) const fn source_owner(self) -> NodeKey {
        self.owner
    }
    pub(in crate::resolve) fn allows(self, feature: KnownFeature) -> bool {
        self.consumers & feature.mask() != 0
    }
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
pub(in crate::resolve) fn checked(
    package: &Package,
    key: NodeKey,
) -> Result<Option<CheckedHeader<'_>>, Error> {
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
    if let Some(context) = package.macro_context().map_err(error)? {
        let mut state = context
            .contracts()
            .0
            .try_borrow_mut()
            .map_err(|_| error("reentrant source macro contract query"))?;
        state.charge(1, 0).map_err(error)?;
        if let Some((declaration, cost)) = state
            .declarations
            .get(&owner.node)
            .map(|entry| (entry.declaration(), entry.work()))
        {
            state.charge(cost, 0).map_err(error)?;
            if declaration.owner != owner.key
                || declaration.attributes != (owner.attributes.start, owner.attributes.end)
                || !is_macro(owner.unit, owner.node)
                || state
                    .declarations
                    .get(&owner.node)
                    .is_none_or(|entry| !entry.valid(owner.unit))
            {
                return Err(error("source macro attribute owner changed"));
            }
            require_features(package, declaration).map_err(error)?;
        } else if let Some(entry) = lists::collect(package, &owner, &mut state)? {
            state
                .charge(1, 4 * size_of::<(NodeKey, Entry)>())
                .map_err(error)?;
            state
                .declarations
                .try_reserve(1)
                .map_err(|_| error("source macro contract allocation failed"))?;
            let bytes = entry.payload();
            state.payload(bytes).map_err(error)?;
            state.witness_bytes = state.witness_bytes.saturating_add(bytes);
            state.declarations.insert(entry.declaration().node, entry);
        }
        Ok(state
            .declarations
            .get(&owner.node)
            .map(|entry| CheckedHeader::new(package, entry)))
    } else {
        Ok(lists::collect(package, &owner, &mut State::default())?
            .as_ref()
            .map(|entry| CheckedHeader::new(package, entry)))
    }
}
#[cfg(test)]
pub(in crate::resolve) fn declaration(
    package: &Package,
    key: NodeKey,
) -> Result<Option<Declaration>, Error> {
    Ok(checked(package, key)?.map(|header| header.summary()))
}
/// Only a private expansion owner or a borrowed immutable report session may
/// consume a published snapshot without repeating its source authentication.
pub(in crate::resolve) fn published(
    package: &Package,
    key: NodeKey,
    expected: Option<&Rc<HeaderData>>,
    steps: usize,
) -> Result<Declaration, Error> {
    let error = |message| Error::at(key.file, 0, message);
    let context = package
        .macro_context()
        .map_err(error)?
        .ok_or_else(|| error("missing source macro provider"))?;
    let mut state = context
        .contracts()
        .0
        .try_borrow_mut()
        .map_err(|_| error("reentrant published macro query"))?;
    state.charge(steps, 0).map_err(error)?;
    let actual = state
        .declarations
        .get(&key)
        .ok_or_else(|| error("unknown published macro header"))?;
    let matches = match (actual, expected) {
        (Entry::Plain(_), None) => true,
        (Entry::Owned(actual), Some(expected)) => Rc::ptr_eq(actual, expected),
        _ => false,
    };
    if !matches {
        return Err(error("published macro header owner changed"));
    }
    let declaration = actual.declaration();
    if declaration.node != key {
        return Err(error("published macro header node changed"));
    }
    state.charge(2, 0).map_err(error)?;
    let current = owner(package, declaration.source_owner())?
        .ok_or_else(|| error("published macro owner is inactive"))?;
    if current.key != declaration.owner
        || current.node != declaration.node
        || (current.attributes.start, current.attributes.end) != declaration.attributes
        || !is_macro(current.unit, current.node)
    {
        return Err(error("published macro owner or attribute range changed"));
    }
    require_features(package, declaration).map_err(error)?;
    Ok(declaration)
}
#[cfg(test)]
pub(in crate::resolve) fn existing(
    package: &Package,
    key: NodeKey,
) -> Result<Option<Declaration>, Error> {
    let context = package
        .macro_context()
        .map_err(|message| Error::at(key.file, 0, message))?
        .ok_or_else(|| Error::at(key.file, 0, "missing source macro provider"))?;
    let owner = {
        let mut state = context
            .contracts()
            .0
            .try_borrow_mut()
            .map_err(|_| Error::at(key.file, 0, "reentrant source macro contract query"))?;
        state
            .charge(1, 0)
            .map_err(|message| Error::at(key.file, 0, message))?;
        state
            .declarations
            .get(&key)
            .map(|entry| entry.declaration().owner)
    };
    owner
        .map(|owner| declaration(package, owner))
        .transpose()
        .map(Option::flatten)
}
fn require_features(package: &Package, entry: Declaration) -> Result<(), &'static str> {
    package
        .macro_context()?
        .ok_or("internal macro contracts require an actual source provider")?
        .require_header(entry)
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
impl Contracts {
    #[cfg(test)]
    pub(in crate::resolve) fn usage(&self) -> (usize, usize, usize) {
        let state = self.0.borrow();
        (state.steps, state.bytes, state.witness_bytes)
    }
    pub(in crate::resolve) fn memo_payload(&self, bytes: usize) -> Result<(), &'static str> {
        self.0
            .try_borrow_mut()
            .map_err(|_| "reentrant macro memo accounting")?
            .payload(bytes)
    }
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
