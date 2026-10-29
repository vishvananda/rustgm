//! Shared variance products and reverse dependencies, without expanding nominal fields.
use super::Variance;
use crate::support::{
    hash::{HashMap, HashSet},
    retained::Payment,
};

const LIMIT: usize = 1_000_000;
const LITERALS: [Variance; 4] = [
    Variance::Bivariant,
    Variance::Covariant,
    Variance::Contravariant,
    Variance::Invariant,
];
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct Term(usize);
impl Term {
    pub(super) const fn constant(value: Variance) -> Self {
        Self(match value {
            Variance::Bivariant => 0,
            Variance::Covariant => 1,
            Variance::Contravariant => 2,
            Variance::Invariant => 3,
        })
    }
}
struct Node {
    value: Variance,
    product: Option<(Term, Term)>,
    head: Option<usize>,
    pending: bool,
}
impl Node {
    fn new(value: Variance) -> Self {
        Self {
            value,
            product: None,
            head: None,
            pending: false,
        }
    }
}
#[derive(Clone, Copy)]
struct Edge {
    target: Term,
    next: Option<usize>,
}
pub(super) struct Solver<'a> {
    payment: Option<Payment<'a>>,
    nodes: Vec<Node>,
    products: HashMap<(Term, Term), Term>,
    edges: Vec<Edge>,
    queue: Vec<Term>,
    pub(super) steps: usize,
}
impl Default for Solver<'_> {
    fn default() -> Self {
        Self {
            payment: None,
            nodes: LITERALS.into_iter().map(Node::new).collect(),
            products: HashMap::default(),
            edges: Vec::new(),
            queue: Vec::new(),
            steps: 0,
        }
    }
}
impl<'a> Solver<'a> {
    pub(super) fn paid(payment: Payment<'a>) -> Result<Self, &'static str> {
        let mut solver = Self {
            payment: Some(payment),
            nodes: Vec::new(),
            products: HashMap::default(),
            edges: Vec::new(),
            queue: Vec::new(),
            steps: 0,
        };
        payment(LITERALS.len(), 0)?;
        super::payments::vector(Some(payment), &mut solver.nodes, LITERALS.len())?;
        solver.nodes.extend(LITERALS.map(Node::new));
        Ok(solver)
    }
    pub(super) fn pay(&self, work: usize, bytes: usize) -> Result<(), &'static str> {
        self.payment.map_or(Ok(()), |pay| pay(work, bytes))
    }
    pub(super) fn vector<T>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), &'static str> {
        super::payments::vector(self.payment, values, additional)
    }
    pub(super) fn map<K: Eq + std::hash::Hash, V>(
        &self,
        values: &mut HashMap<K, V>,
        additional: usize,
    ) -> Result<(), &'static str> {
        super::payments::map(self.payment, values, additional)
    }
    pub(super) fn insert<K: Eq + std::hash::Hash>(
        &self,
        values: &mut HashSet<K>,
        key: K,
    ) -> Result<bool, &'static str> {
        super::payments::insert(self.payment, values, key)
    }
    pub(super) fn charge(&mut self) -> Result<(), &'static str> {
        self.pay(1, 0)?;
        if self.steps >= LIMIT {
            return Err("nominal variance work budget exceeded");
        }
        self.steps += 1;
        Ok(())
    }
    pub(super) fn value(&self, term: Term) -> Variance {
        self.nodes[term.0].value
    }
    pub(super) fn variable(&mut self, value: Variance) -> Result<Term, &'static str> {
        self.charge()?;
        let term = Term(self.nodes.len());
        super::payments::vector(self.payment, &mut self.nodes, 1)?;
        self.nodes.push(Node::new(value));
        Ok(term)
    }
    pub(super) fn compose(&mut self, a: Term, b: Term) -> Result<Term, &'static str> {
        self.charge()?;
        // Only literal constants simplify: a currently bivariant variable can change.
        let co = Term::constant(Variance::Covariant);
        let bi = Term::constant(Variance::Bivariant);
        if a == bi || b == bi {
            return Ok(bi);
        }
        if a == co || b == co {
            return Ok(if a == co { b } else { a });
        }
        if a.0 < 4 && b.0 < 4 {
            return Ok(Term::constant(self.value(a).compose(self.value(b))));
        }
        let key = if a <= b { (a, b) } else { (b, a) };
        if let Some(&term) = self.products.get(&key) {
            return Ok(term);
        }
        let term = self.variable(self.value(a).compose(self.value(b)))?;
        self.nodes[term.0].product = Some(key);
        super::payments::map(self.payment, &mut self.products, 1)?;
        self.products.insert(key, term);
        self.edge(a, term)?;
        if a != b {
            self.edge(b, term)?;
        }
        Ok(term)
    }
    fn edge(&mut self, source: Term, target: Term) -> Result<(), &'static str> {
        self.charge()?;
        super::payments::vector(self.payment, &mut self.edges, 1)?;
        let index = self.edges.len();
        self.edges.push(Edge {
            target,
            next: self.nodes[source.0].head,
        });
        self.nodes[source.0].head = Some(index);
        Ok(())
    }
    fn update(&mut self, target: Term, value: Variance) -> Result<(), &'static str> {
        let node = &self.nodes[target.0];
        let next = node.value.join(value);
        if next != node.value {
            let enqueue = !node.pending;
            if enqueue {
                super::payments::vector(self.payment, &mut self.queue, 1)?;
            }
            let node = &mut self.nodes[target.0];
            node.value = next;
            node.pending = true;
            if enqueue {
                self.queue.push(target);
            }
        }
        Ok(())
    }
    pub(super) fn constrain(&mut self, target: Term, source: Term) -> Result<(), &'static str> {
        if source.0 < 4 {
            self.charge()?;
        } else {
            self.edge(source, target)?;
        }
        self.update(target, self.value(source))
    }
    pub(super) fn solve(&mut self) -> Result<(), &'static str> {
        while let Some(source) = self.queue.pop() {
            self.charge()?;
            self.nodes[source.0].pending = false;
            let mut next = self.nodes[source.0].head;
            while let Some(index) = next {
                self.charge()?;
                let edge = self.edges[index];
                let value = self.nodes[edge.target.0].product.map_or_else(
                    || self.value(source),
                    |(a, b)| self.value(a).compose(self.value(b)),
                );
                self.update(edge.target, value)?;
                next = edge.next;
            }
        }
        Ok(())
    }
}
