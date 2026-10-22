//! Reduced binary decisions over scalar ranks; updates share unchanged domain subtrees.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(u32);
impl Id {
    pub const EMPTY: Self = Self(0);
    pub const ALL: Self = Self(1);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Variable(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    Union,
    Intersection,
    Difference,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Query {
    operation: Operation,
    left: Id,
    right: Id,
}
impl Query {
    /// Identities stay below 2^31 (see [`Graph::intern`]); two bits name the operation.
    fn packed(self) -> u64 {
        (self.operation as u64) << 62 | u64::from(self.left.0) << 31 | u64::from(self.right.0)
    }
    fn new(operation: Operation, mut left: Id, mut right: Id) -> Self {
        if operation != Operation::Difference && left.0 > right.0 {
            std::mem::swap(&mut left, &mut right);
        }
        Self {
            operation,
            left,
            right,
        }
    }
}
/// Variables sort by typed-place introduction, then most significant bit first.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Position {
    variable: Variable,
    level: u8,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    position: Position,
    zero: Id,
    one: Id,
}
impl Node {
    fn packed(self) -> u128 {
        u128::from(self.position.variable.0) << 72
            | u128::from(self.position.level) << 64
            | u128::from(self.zero.0) << 32
            | u128::from(self.one.0)
    }
}
/// Packed unique and computed tables: open addressing with linear probing,
/// insert-only between clears, at most half full.
trait Packed: Copy + Eq + Default {
    fn spread(self) -> u64;
}
impl Packed for u64 {
    fn spread(self) -> u64 {
        let mixed = (self ^ (self >> 31)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        mixed ^ (mixed >> 29)
    }
}
impl Packed for u128 {
    fn spread(self) -> u64 {
        #[expect(clippy::cast_possible_truncation, reason = "fold both 64-bit halves")]
        let (low, high) = (self as u64, (self >> 64) as u64);
        (low ^ high.wrapping_mul(0xbf58_476d_1ce4_e5b9)).spread()
    }
}
const VACANT: u32 = u32::MAX;
#[derive(Clone, Copy, Default)]
struct Slot<K> {
    key: K,
    value: u32,
}
#[derive(Default)]
struct Table<K> {
    slots: Vec<Slot<K>>,
    len: usize,
}
impl<K: Packed> Table<K> {
    fn get(&self, key: K) -> Option<Id> {
        let mask = self.slots.len().checked_sub(1)?;
        #[expect(clippy::cast_possible_truncation, reason = "masked probe index")]
        let mut index = key.spread() as usize & mask;
        loop {
            let slot = self.slots[index];
            if slot.value == VACANT {
                return None;
            }
            if slot.key == key {
                return Some(Id(slot.value));
            }
            index = (index + 1) & mask;
        }
    }
    fn insert(&mut self, key: K, value: Id) {
        if self.len.saturating_add(1).saturating_mul(2) > self.slots.len() {
            self.grow();
        }
        let mask = self.slots.len() - 1;
        #[expect(clippy::cast_possible_truncation, reason = "masked probe index")]
        let mut index = key.spread() as usize & mask;
        loop {
            let slot = &mut self.slots[index];
            if slot.value == VACANT {
                *slot = Slot {
                    key,
                    value: value.0,
                };
                self.len += 1;
                return;
            }
            if slot.key == key {
                slot.value = value.0;
                return;
            }
            index = (index + 1) & mask;
        }
    }
    fn grow(&mut self) {
        let capacity = self.slots.len().saturating_mul(2).max(16);
        let previous = std::mem::replace(
            &mut self.slots,
            vec![
                Slot {
                    key: K::default(),
                    value: VACANT,
                };
                capacity
            ],
        );
        self.len = 0;
        for slot in previous {
            if slot.value != VACANT {
                self.insert(slot.key, Id(slot.value));
            }
        }
    }
    fn clear(&mut self) {
        if self.slots.len() > RETAINED_ENTRIES.saturating_mul(2) {
            self.slots = Vec::new();
        } else {
            self.slots.fill(Slot {
                key: K::default(),
                value: VACANT,
            });
        }
        self.len = 0;
    }
}
struct Partition {
    position: Position,
    zero: Query,
    one: Query,
}
enum Work {
    Enter(Query),
    Finish(Query, Partition),
}
enum RangeWork {
    Enter { lower: u128, upper: u128, level: u8 },
    Finish(Position),
}
/// Retained tables larger than this are released rather than cleared, so one
/// large match cannot make every later small match pay for its capacity.
const RETAINED_ENTRIES: usize = 1 << 12;

#[derive(Default)]
pub struct Graph {
    maxima: Vec<u128>,
    nodes: Vec<Node>,
    index: Table<u128>,
    queries: Table<u64>,
    /// Reusable traversal stacks; each operation leaves them empty.
    stack: Vec<Work>,
    ranges: Vec<RangeWork>,
    values: Vec<Id>,
    work: u64,
    limit: u64,
}
impl Graph {
    #[cfg(test)]
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            ..Self::default()
        }
    }
    /// Begin an independent decision space while retaining bounded storage.
    pub fn reset(&mut self, limit: u64) {
        self.maxima.clear();
        self.nodes.clear();
        self.index.clear();
        self.queries.clear();
        self.stack.clear();
        self.ranges.clear();
        self.values.clear();
        self.work = 0;
        self.limit = limit;
    }
    pub fn charge(&mut self) -> Result<(), &'static str> {
        self.work = self
            .work
            .checked_add(1)
            .ok_or("pattern coverage work overflow")?;
        if self.work > self.limit {
            return Err("pattern coverage work budget exceeded");
        }
        Ok(())
    }
    pub fn work(&self) -> u64 {
        self.work
    }
    pub fn variable(&mut self, maximum: u128) -> Result<Variable, &'static str> {
        self.charge()?;
        let id =
            Variable(u32::try_from(self.maxima.len()).map_err(|_| "too many coverage variables")?);
        self.maxima.push(maximum);
        Ok(id)
    }
    pub fn interval(
        &mut self,
        variable: Variable,
        lower: u128,
        mut upper: u128,
    ) -> Result<Id, &'static str> {
        let maximum = self.maxima[variable.0 as usize];
        if lower > upper || upper > maximum {
            return Err("invalid coverage interval");
        }
        if lower == 0 && upper == maximum {
            self.charge()?;
            return Ok(Id::ALL);
        }
        let leading = maximum.leading_zeros();
        let ceiling = u128::MAX >> leading;
        // Unused bit encodings alias the last valid rank. Every atomic predicate
        // and every set operation then agrees on those encodings; complement
        // cannot invent an out-of-domain value. Witnesses clamp back to maximum.
        if upper == maximum {
            upper = ceiling;
        }
        let level = u8::try_from(leading).map_err(|_| "invalid scalar width")?;
        let mut work = std::mem::take(&mut self.ranges);
        let mut values = std::mem::take(&mut self.values);
        work.clear();
        values.clear();
        work.push(RangeWork::Enter {
            lower: 0,
            upper: ceiling,
            level,
        });
        let result = self.subdivide(variable, (lower, upper), &mut work, &mut values);
        work.clear();
        values.clear();
        self.ranges = work;
        self.values = values;
        result
    }
    fn subdivide(
        &mut self,
        variable: Variable,
        (lower, upper): (u128, u128),
        work: &mut Vec<RangeWork>,
        values: &mut Vec<Id>,
    ) -> Result<Id, &'static str> {
        while let Some(next) = work.pop() {
            self.charge()?;
            match next {
                RangeWork::Enter {
                    lower: start,
                    upper: end,
                    level,
                } => {
                    if end < lower || start > upper {
                        values.push(Id::EMPTY);
                    } else if start >= lower && end <= upper {
                        values.push(Id::ALL);
                    } else {
                        if start == end || level >= 128 {
                            return Err("invalid interval subdivision");
                        }
                        let middle = start + (end - start) / 2;
                        work.push(RangeWork::Finish(Position { variable, level }));
                        work.push(RangeWork::Enter {
                            lower: middle + 1,
                            upper: end,
                            level: level + 1,
                        });
                        work.push(RangeWork::Enter {
                            lower: start,
                            upper: middle,
                            level: level + 1,
                        });
                    }
                }
                RangeWork::Finish(position) => {
                    let one = values.pop().ok_or("missing upper interval branch")?;
                    let zero = values.pop().ok_or("missing lower interval branch")?;
                    values.push(self.intern(Node {
                        position,
                        zero,
                        one,
                    })?);
                }
            }
        }
        values.pop().ok_or("missing interval result")
    }
    fn node(&self, id: Id) -> Option<Node> {
        id.0.checked_sub(2)
            .and_then(|index| self.nodes.get(index as usize))
            .copied()
    }
    fn intern(&mut self, node: Node) -> Result<Id, &'static str> {
        self.charge()?;
        if node.zero == node.one {
            return Ok(node.zero);
        }
        if let Some(id) = self.index.get(node.packed()) {
            return Ok(id);
        }
        // Packed table keys require identities below 2^31.
        let id = Id(u32::try_from(self.nodes.len())
            .ok()
            .and_then(|n| n.checked_add(2))
            .filter(|&n| n < 1 << 31)
            .ok_or("too many coverage nodes")?);
        self.nodes.push(node);
        self.index.insert(node.packed(), id);
        Ok(id)
    }
    fn simple(query: Query) -> Option<Id> {
        let Query {
            operation,
            left,
            right,
        } = query;
        match operation {
            Operation::Union if left == Id::ALL || right == Id::ALL => Some(Id::ALL),
            Operation::Union if left == Id::EMPTY || left == right => Some(right),
            Operation::Intersection if left == Id::EMPTY || right == Id::EMPTY => Some(Id::EMPTY),
            Operation::Intersection if left == Id::ALL || left == right => Some(right),
            Operation::Intersection if right == Id::ALL => Some(left),
            Operation::Difference if left == Id::EMPTY || right == Id::ALL || left == right => {
                Some(Id::EMPTY)
            }
            Operation::Difference if right == Id::EMPTY => Some(left),
            _ => None,
        }
    }
    fn split(&self, query: Query) -> Result<Partition, &'static str> {
        let left = self.node(query.left);
        let right = self.node(query.right);
        let position = left
            .into_iter()
            .chain(right)
            .map(|n| n.position)
            .min()
            .ok_or("unreduced terminal coverage query")?;
        let left = left.filter(|n| n.position == position);
        let right = right.filter(|n| n.position == position);
        Ok(Partition {
            position,
            zero: Query::new(
                query.operation,
                left.map_or(query.left, |n| n.zero),
                right.map_or(query.right, |n| n.zero),
            ),
            one: Query::new(
                query.operation,
                left.map_or(query.left, |n| n.one),
                right.map_or(query.right, |n| n.one),
            ),
        })
    }
    pub fn apply(&mut self, operation: Operation, left: Id, right: Id) -> Result<Id, &'static str> {
        let root = Query::new(operation, left, right);
        self.charge()?;
        if let Some(result) = Self::simple(root) {
            return Ok(result);
        }
        if let Some(result) = self.queries.get(root.packed()) {
            return Ok(result);
        }
        let mut work = std::mem::take(&mut self.stack);
        work.clear();
        work.push(Work::Enter(root));
        let result = self.solve(&mut work);
        work.clear();
        self.stack = work;
        result?;
        self.queries
            .get(root.packed())
            .ok_or("missing coverage result")
    }
    fn solve(&mut self, work: &mut Vec<Work>) -> Result<(), &'static str> {
        while let Some(next) = work.pop() {
            self.charge()?;
            match next {
                Work::Enter(query) => {
                    if self.queries.get(query.packed()).is_some() {
                        continue;
                    }
                    if let Some(result) = Self::simple(query) {
                        self.queries.insert(query.packed(), result);
                        continue;
                    }
                    let partition = self.split(query)?;
                    let (zero, one) = (partition.zero, partition.one);
                    work.push(Work::Finish(query, partition));
                    work.push(Work::Enter(one));
                    work.push(Work::Enter(zero));
                }
                Work::Finish(query, partition) => {
                    let zero = self
                        .queries
                        .get(partition.zero.packed())
                        .ok_or("missing lower coverage subproblem")?;
                    let one = self
                        .queries
                        .get(partition.one.packed())
                        .ok_or("missing upper coverage subproblem")?;
                    let result = self.intern(Node {
                        position: partition.position,
                        zero,
                        one,
                    })?;
                    self.queries.insert(query.packed(), result);
                }
            }
        }
        Ok(())
    }
    pub fn witness(&mut self, mut id: Id) -> Result<Vec<(Variable, u128)>, &'static str> {
        let mut values: Vec<(Variable, u128)> = Vec::new();
        while let Some(node) = self.node(id) {
            self.charge()?;
            if values
                .last()
                .is_none_or(|(variable, _)| *variable != node.position.variable)
            {
                values.push((node.position.variable, 0));
            }
            if node.zero == Id::EMPTY {
                let (_, rank) = values
                    .last_mut()
                    .ok_or("missing coverage witness variable")?;
                *rank |= 1_u128 << (127 - node.position.level);
                id = node.one;
            } else {
                id = node.zero;
            }
        }
        if id != Id::ALL {
            return Err("cannot witness empty coverage");
        }
        for (variable, rank) in &mut values {
            *rank = (*rank).min(self.maxima[variable.0 as usize]);
        }
        Ok(values)
    }
    #[cfg(test)]
    pub(super) fn contains(&self, mut id: Id, values: &[u128]) -> Result<bool, &'static str> {
        while let Some(node) = self.node(id) {
            let value = *values
                .get(node.position.variable.0 as usize)
                .ok_or("missing test variable")?;
            id = if value & (1_u128 << (127 - node.position.level)) == 0 {
                node.zero
            } else {
                node.one
            };
        }
        Ok(id == Id::ALL)
    }
}
