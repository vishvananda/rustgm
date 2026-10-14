//! Monotone Boolean equations: alternatives activate once; products need both inputs.
use std::collections::HashMap;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Term(usize);
impl Term {
    pub const ALWAYS: Self = Self(0);
}
struct Node {
    ready: bool,
    remaining: u8,
    dependents: Vec<Term>,
}
pub(super) struct Conditions {
    nodes: Vec<Node>,
    products: HashMap<(Term, Term), Term>,
    pending: Vec<Term>,
    pub steps: usize,
}
impl Default for Conditions {
    fn default() -> Self {
        Self {
            nodes: vec![Node {
                ready: true,
                remaining: 0,
                dependents: Vec::new(),
            }],
            products: HashMap::new(),
            pending: vec![Term::ALWAYS],
            steps: 0,
        }
    }
}
impl Conditions {
    pub fn charge(&mut self, amount: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(amount)
            .filter(|n| *n <= 1_000_000)
            .ok_or("aggregate storage graph budget exceeded")?;
        Ok(())
    }
    pub fn variable(&mut self) -> Result<Term, &'static str> {
        self.charge(1)?;
        let id = Term(self.nodes.len());
        self.nodes.push(Node {
            ready: false,
            remaining: 0,
            dependents: Vec::new(),
        });
        Ok(id)
    }
    pub fn require(&mut self, target: Term, source: Term) -> Result<(), &'static str> {
        self.charge(1)?;
        self.nodes[source.0].dependents.push(target);
        Ok(())
    }
    pub fn product(&mut self, a: Term, b: Term) -> Result<Term, &'static str> {
        if a == b || b == Term::ALWAYS {
            return Ok(a);
        }
        if a == Term::ALWAYS {
            return Ok(b);
        }
        let pair = if a.0 < b.0 { (a, b) } else { (b, a) };
        if let Some(&id) = self.products.get(&pair) {
            return Ok(id);
        }
        let id = self.variable()?;
        self.nodes[id.0].remaining = 2;
        self.require(id, a)?;
        self.require(id, b)?;
        self.products.insert(pair, id);
        Ok(id)
    }
    pub fn solve(&mut self) -> Result<(), &'static str> {
        while let Some(source) = self.pending.pop() {
            self.charge(1)?;
            for index in 0..self.nodes[source.0].dependents.len() {
                self.charge(1)?;
                let target = self.nodes[source.0].dependents[index];
                let node = &mut self.nodes[target.0];
                if node.ready {
                    continue;
                }
                node.remaining = node.remaining.saturating_sub(1);
                if node.remaining == 0 {
                    node.ready = true;
                    self.pending.push(target);
                }
            }
        }
        Ok(())
    }
    pub fn value(&self, term: Term) -> bool {
        self.nodes[term.0].ready
    }
}
