//! Shared variance products and reverse dependencies, without expanding nominal fields.
use super::Variance;
use std::collections::HashMap;

const LIMIT: usize = 1_000_000;
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
#[derive(Clone, Copy)]
struct Edge {
    target: Term,
    next: Option<usize>,
}
pub(super) struct Solver {
    nodes: Vec<Node>,
    products: HashMap<(Term, Term), Term>,
    edges: Vec<Edge>,
    queue: Vec<Term>,
    pub(super) steps: usize,
}
impl Default for Solver {
    fn default() -> Self {
        Self {
            nodes: [
                Variance::Bivariant,
                Variance::Covariant,
                Variance::Contravariant,
                Variance::Invariant,
            ]
            .into_iter()
            .map(|value| Node {
                value,
                product: None,
                head: None,
                pending: false,
            })
            .collect(),
            products: HashMap::new(),
            edges: Vec::new(),
            queue: Vec::new(),
            steps: 0,
        }
    }
}
impl Solver {
    pub(super) fn charge(&mut self) -> Result<(), &'static str> {
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
        self.nodes.push(Node {
            value,
            product: None,
            head: None,
            pending: false,
        });
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
        self.products.insert(key, term);
        self.edge(a, term)?;
        if a != b {
            self.edge(b, term)?;
        }
        Ok(term)
    }
    fn edge(&mut self, source: Term, target: Term) -> Result<(), &'static str> {
        self.charge()?;
        let index = self.edges.len();
        self.edges.push(Edge {
            target,
            next: self.nodes[source.0].head,
        });
        self.nodes[source.0].head = Some(index);
        Ok(())
    }
    fn update(&mut self, target: Term, value: Variance) {
        let node = &mut self.nodes[target.0];
        let next = node.value.join(value);
        if next != node.value {
            node.value = next;
            if !std::mem::replace(&mut node.pending, true) {
                self.queue.push(target);
            }
        }
    }
    pub(super) fn constrain(&mut self, target: Term, source: Term) -> Result<(), &'static str> {
        if source.0 < 4 {
            self.charge()?;
        } else {
            self.edge(source, target)?;
        }
        self.update(target, self.value(source));
        Ok(())
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
                self.update(edge.target, value);
                next = edge.next;
            }
        }
        Ok(())
    }
}
