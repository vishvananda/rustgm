//! Reduced binary decisions over scalar ranks; updates share unchanged domain subtrees.
use std::collections::HashMap;

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
pub struct Graph {
    maxima: Vec<u128>,
    nodes: Vec<Node>,
    index: HashMap<Node, Id>,
    queries: HashMap<Query, Id>,
    work: u64,
    limit: u64,
}
impl Graph {
    pub fn new(limit: u64) -> Self {
        Self {
            maxima: Vec::new(),
            nodes: Vec::new(),
            index: HashMap::new(),
            queries: HashMap::new(),
            work: 0,
            limit,
        }
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
        let mut work = vec![RangeWork::Enter {
            lower: 0,
            upper: ceiling,
            level: u8::try_from(leading).map_err(|_| "invalid scalar width")?,
        }];
        let mut values = Vec::new();
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
        if let Some(&id) = self.index.get(&node) {
            return Ok(id);
        }
        let id = Id(u32::try_from(self.nodes.len())
            .ok()
            .and_then(|n| n.checked_add(2))
            .ok_or("too many coverage nodes")?);
        self.nodes.push(node);
        self.index.insert(node, id);
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
        if let Some(&result) = self.queries.get(&root) {
            return Ok(result);
        }
        let mut work = vec![Work::Enter(root)];
        while let Some(next) = work.pop() {
            self.charge()?;
            match next {
                Work::Enter(query) => {
                    if self.queries.contains_key(&query) {
                        continue;
                    }
                    if let Some(result) = Self::simple(query) {
                        self.queries.insert(query, result);
                        continue;
                    }
                    let partition = self.split(query)?;
                    let (zero, one) = (partition.zero, partition.one);
                    work.push(Work::Finish(query, partition));
                    work.push(Work::Enter(one));
                    work.push(Work::Enter(zero));
                }
                Work::Finish(query, partition) => {
                    let zero = *self
                        .queries
                        .get(&partition.zero)
                        .ok_or("missing lower coverage subproblem")?;
                    let one = *self
                        .queries
                        .get(&partition.one)
                        .ok_or("missing upper coverage subproblem")?;
                    let result = self.intern(Node {
                        position: partition.position,
                        zero,
                        one,
                    })?;
                    self.queries.insert(query, result);
                }
            }
        }
        self.queries
            .get(&root)
            .copied()
            .ok_or("missing coverage result")
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
