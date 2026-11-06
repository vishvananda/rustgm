//! Canonical radix maps share matching requirements without copying growing prefixes.
use crate::support::hash::HashMap;
use crate::support::retained::Growth;
use std::hash::Hash;
#[cfg(test)]
#[path = "../../tests/invariants/outlives_maps.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Identity,
    Budget,
}
impl std::fmt::Display for Error {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(match self {
            Self::Identity => "invalid persistent map identity",
            Self::Budget => "persistent map work budget exceeded",
        })
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Id(u32);
impl Id {
    pub const EMPTY: Self = Self(0);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Node<V> {
    Leaf(u32, V),
    Branch {
        prefix: u32,
        height: u8,
        left: Id,
        right: Id,
    },
}
impl<V: Copy> Node<V> {
    fn prefix(self) -> u32 {
        match self {
            Self::Leaf(key, _) | Self::Branch { prefix: key, .. } => key,
        }
    }
    fn height(self) -> u8 {
        match self {
            Self::Leaf(..) => 0,
            Self::Branch { height, .. } => height,
        }
    }
}

pub(crate) struct Map<V> {
    nodes: Vec<Node<V>>,
    index: HashMap<Node<V>, Id>,
    unions: HashMap<(Id, Id), Option<Id>>,
    pub steps: usize,
}
impl<V> Default for Map<V> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            index: HashMap::default(),
            unions: HashMap::default(),
            steps: 0,
        }
    }
}
impl<V: Copy + Eq + Hash> Map<V> {
    fn charge<const CHARGED: bool>(
        &mut self,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if CHARGED {
            pay(1, 0)?;
        }
        self.steps = self.steps.saturating_add(1);
        if self.steps > 1_000_000 {
            return Err(Error::Budget);
        }
        Ok(())
    }
    pub fn node(&self, id: Id) -> Result<Node<V>, Error> {
        self.nodes
            .get(id.0.checked_sub(1).ok_or(Error::Identity)? as usize)
            .copied()
            .ok_or(Error::Identity)
    }
    fn intern<const CHARGED: bool>(
        &mut self,
        node: Node<V>,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<Id, Error> {
        self.charge::<CHARGED>(pay)?;
        if let Some(&id) = self.index.get(&node) {
            return Ok(id);
        }
        let id = Id(u32::try_from(self.nodes.len() + 1).map_err(|_| Error::Budget)?);
        if CHARGED {
            let mut growth = Growth::vector::<Node<V>>(self.nodes.len(), self.nodes.capacity(), 1);
            growth.include(Growth::map::<Node<V>, Id>(
                self.index.len(),
                self.index.capacity(),
                1,
            ));
            pay(growth.work, growth.bytes)?;
        }
        self.nodes.push(node);
        self.index.insert(node, id);
        Ok(id)
    }
    pub fn singleton(&mut self, key: u32, value: V) -> Result<Id, Error> {
        self.intern::<false>(Node::Leaf(key, value), &|_, _| Ok(()))
    }
    pub(crate) fn singleton_charged(
        &mut self,
        key: u32,
        value: V,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<Id, Error> {
        self.intern::<true>(Node::Leaf(key, value), pay)
    }
    pub fn get(&mut self, mut root: Id, key: u32) -> Result<Option<V>, Error> {
        while root != Id::EMPTY {
            self.charge::<false>(&|_, _| Ok(()))?;
            match self.node(root)? {
                Node::Leaf(found, value) => return Ok((found == key).then_some(value)),
                Node::Branch {
                    prefix,
                    height,
                    left,
                    right,
                } => {
                    if prefix_of(key, height) != prefix {
                        return Ok(None);
                    }
                    root = if high(key, height) { right } else { left };
                }
            }
        }
        Ok(None)
    }
    fn branch<const CHARGED: bool>(
        &mut self,
        height: u8,
        left: Id,
        right: Id,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<Id, Error> {
        if left == Id::EMPTY {
            return Ok(right);
        }
        if right == Id::EMPTY {
            return Ok(left);
        }
        let a = self.node(left)?;
        let b = self.node(right)?;
        if height == 0
            || height > 32
            || a.height() >= height
            || b.height() >= height
            || high(a.prefix(), height)
            || !high(b.prefix(), height)
            || prefix_of(a.prefix(), height) != prefix_of(b.prefix(), height)
        {
            return Err(Error::Identity);
        }
        self.intern::<CHARGED>(
            Node::Branch {
                prefix: prefix_of(a.prefix(), height),
                height,
                left,
                right,
            },
            pay,
        )
    }
    /// A conflicting value rejects the complete match; no trial binding is committed.
    /// Recursion consumes a radix level at each descent and is bounded by 32.
    pub fn union(&mut self, a: Id, b: Id) -> Result<Option<Id>, Error> {
        self.union_observed::<false>(a, b, &|_, _| Ok(()))
    }
    pub(crate) fn union_charged(
        &mut self,
        a: Id,
        b: Id,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<Option<Id>, Error> {
        self.union_observed::<true>(a, b, pay)
    }
    fn union_observed<const CHARGED: bool>(
        &mut self,
        a: Id,
        b: Id,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<Option<Id>, Error> {
        self.charge::<CHARGED>(pay)?;
        if a == b || b == Id::EMPTY {
            return Ok(Some(a));
        }
        if a == Id::EMPTY {
            return Ok(Some(b));
        }
        let key = if a.0 < b.0 { (a, b) } else { (b, a) };
        if let Some(&found) = self.unions.get(&key) {
            return Ok(found);
        }
        let result = self.combine::<CHARGED>(a, b, pay)?;
        if CHARGED {
            let growth =
                Growth::map::<(Id, Id), Option<Id>>(self.unions.len(), self.unions.capacity(), 1);
            pay(growth.work, growth.bytes)?;
        }
        self.unions.insert(key, result);
        Ok(result)
    }
    fn combine<const CHARGED: bool>(
        &mut self,
        a: Id,
        b: Id,
        pay: &impl Fn(usize, usize) -> Result<(), Error>,
    ) -> Result<Option<Id>, Error> {
        let left = self.node(a)?;
        let right = self.node(b)?;
        let divergence = u8::try_from(32 - (left.prefix() ^ right.prefix()).leading_zeros())
            .map_err(|_| Error::Budget)?;
        if divergence > left.height().max(right.height()) {
            let (low, high) = if high(left.prefix(), divergence) {
                (b, a)
            } else {
                (a, b)
            };
            return self.branch::<CHARGED>(divergence, low, high, pay).map(Some);
        }
        if left.height() < right.height() {
            return self.combine::<CHARGED>(b, a, pay);
        }
        let Node::Branch {
            height,
            left: low,
            right: high_child,
            ..
        } = left
        else {
            // Equal keys with equal values have the same interned identity, which
            // union handled above. Two distinct leaves here are a conflict.
            return Ok(None);
        };
        let children = if let Node::Branch {
            left,
            right,
            height: other,
            ..
        } = right
            && height == other
        {
            self.union_observed::<CHARGED>(low, left, pay)?
                .zip(self.union_observed::<CHARGED>(high_child, right, pay)?)
        } else if high(right.prefix(), height) {
            self.union_observed::<CHARGED>(high_child, b, pay)?
                .map(|child| (low, child))
        } else {
            self.union_observed::<CHARGED>(low, b, pay)?
                .map(|child| (child, high_child))
        };
        children
            .map(|(low, high)| self.branch::<CHARGED>(height, low, high, pay))
            .transpose()
    }
}
fn prefix_of(key: u32, height: u8) -> u32 {
    if height == 32 {
        0
    } else {
        key & (u32::MAX << height)
    }
}
fn high(key: u32, height: u8) -> bool {
    key & (1 << (height - 1)) != 0
}
