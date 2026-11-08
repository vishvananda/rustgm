//! Rustc's coherence first asks whether two trait headers could unify at all,
//! with every implementation parameter standing for an inference variable, and
//! instantiates neither header for a pair that cannot. Lifetimes never reject;
//! parameters, projections and opaque types may unify with anything. Only a
//! rigid constructor mismatch rejects, so the complete equation would fail too.
use crate::types::{
    GenericArgs, Kind, Store, TypeId,
    arguments::{Argument, ConstId, ConstKind},
    parameters::predicates::Predicate,
};
/// Rustc's `DeepRejectCtxt::STARTING_DEPTH`: deeper structure may still unify.
const DEPTH: u8 = 8;
#[derive(Clone, Copy)]
enum Pair {
    Type(TypeId, TypeId, u8),
    Const(ConstId, ConstId),
}
struct Walk<'a> {
    store: &'a Store,
    pending: Vec<Pair>,
}
/// Whether two trait headers may unify, and the nodes compared to decide it.
/// Subjects precede trait arguments, as in rustc's `args_may_unify`.
///
/// # Errors
/// Rejects unknown type, constant and dynamic shape identities.
pub(super) fn may_unify(
    store: &Store,
    left: Predicate,
    right: Predicate,
) -> Result<(bool, usize), &'static str> {
    let mut walk = Walk {
        store,
        pending: Vec::new(),
    };
    let application = |predicate: Predicate| {
        predicate
            .bound
            .trait_parts()
            .and_then(|(id, _)| store.trait_application(id))
    };
    match (application(left), application(right)) {
        (Some(x), Some(y)) if x.item == y.item => walk.arguments(x.arguments, y.arguments, DEPTH),
        (None, None) if left.bound == right.bound => {}
        _ => return Ok((true, 1)),
    }
    walk.pending.push(Pair::Type(left.ty, right.ty, DEPTH));
    let mut compared = 0_usize;
    while let Some(pair) = walk.pending.pop() {
        compared = compared.saturating_add(1);
        let possible = match pair {
            Pair::Type(x, y, depth) => walk.types(x, y, depth)?,
            Pair::Const(x, y) => walk.constants(x, y)?,
        };
        if !possible {
            return Ok((false, compared));
        }
    }
    Ok((true, compared))
}
impl Walk<'_> {
    /// Queue argument pairs so that the first is compared first. A list that
    /// cannot be read pairwise is left to the complete equation.
    fn arguments(&mut self, x: GenericArgs, y: GenericArgs, depth: u8) {
        let (Some(xs), Some(ys)) = (self.store.arguments(x), self.store.arguments(y)) else {
            return;
        };
        if xs.len() != ys.len() {
            return;
        }
        let start = self.pending.len();
        for pair in xs.zip(ys) {
            match pair {
                (Argument::Type(p), Argument::Type(q)) => {
                    self.pending.push(Pair::Type(p, q, depth));
                }
                (Argument::Const(p), Argument::Const(q)) => self.pending.push(Pair::Const(p, q)),
                _ => {}
            }
        }
        self.pending[start..].reverse();
    }
    fn list(&mut self, x: crate::types::ListId, y: crate::types::ListId, depth: u8) -> bool {
        let (Some(xs), Some(ys)) = (self.store.list(x), self.store.list(y)) else {
            return true;
        };
        if xs.len() != ys.len() {
            return false;
        }
        self.pending.extend(
            xs.iter()
                .zip(ys)
                .rev()
                .map(|(&p, &q)| Pair::Type(p, q, depth)),
        );
        true
    }
    fn types(&mut self, x: TypeId, y: TypeId, depth: u8) -> Result<bool, &'static str> {
        if x == y {
            return Ok(true);
        }
        let kind = |ty| self.store.kind(ty).ok_or("invalid coherence header type");
        let (left, right) = (kind(x)?, kind(y)?);
        if open(left) || open(right) {
            return Ok(true);
        }
        let Some(depth) = depth.checked_sub(1) else {
            return Ok(true);
        };
        self.structure(left, right, depth)
    }
    /// Rigid constructors agree only with their own variant.
    fn structure(&mut self, left: Kind, right: Kind, depth: u8) -> Result<bool, &'static str> {
        if let (Some((x, p)), Some((y, q))) = (Single::of(left), Single::of(right)) {
            self.push(p, q, depth);
            return Ok(self.agree(x, y));
        }
        Ok(match (left, right) {
            (Kind::Primitive(p), Kind::Primitive(q)) => p == q,
            (Kind::Foreign(p), Kind::Foreign(q)) => p == q,
            (Kind::Never, Kind::Never) => true,
            (Kind::Tuple(p), Kind::Tuple(q)) => self.list(p, q, depth),
            (Kind::Function { .. }, Kind::Function { .. }) => self.signatures(left, right, depth),
            (Kind::Dynamic { shape: p, .. }, Kind::Dynamic { shape: q, .. }) => {
                let principal = |id| {
                    self.store
                        .dynamic_shape(id)
                        .map(|shape| shape.principal)
                        .ok_or("invalid coherence header object shape")
                };
                principal(p)? == principal(q)?
            }
            (Kind::Adt(p, a), Kind::Adt(q, b))
            | (
                Kind::FunctionItem {
                    item: p,
                    arguments: a,
                    ..
                },
                Kind::FunctionItem {
                    item: q,
                    arguments: b,
                    ..
                },
            ) => p == q && self.related(a, b, depth),
            (
                Kind::Closure {
                    identity: p,
                    arguments: a,
                    ..
                },
                Kind::Closure {
                    identity: q,
                    arguments: b,
                    ..
                },
            ) => p == q && self.related(a, b, depth),
            _ => false,
        })
    }
    /// Rustc compares a pointer's mutability and an array's evaluated length.
    fn agree(&mut self, x: Single, y: Single) -> bool {
        match (x, y) {
            (Single::Array(Length::Value(m)), Single::Array(Length::Value(n))) => m == n,
            (Single::Array(Length::Term(m)), Single::Array(Length::Term(n))) => {
                self.pending.push(Pair::Const(m, n));
                true
            }
            (Single::Array(_), Single::Array(_)) => true,
            _ => x == y,
        }
    }
    /// Rustc compares function pointer safety, ABI and arity, then each input
    /// and the output.
    fn signatures(&mut self, left: Kind, right: Kind, depth: u8) -> bool {
        let (
            Kind::Function {
                parameters,
                result,
                unsafe_,
                convention,
                ..
            },
            Kind::Function {
                parameters: other_parameters,
                result: other_result,
                unsafe_: other_unsafe,
                convention: other_convention,
                ..
            },
        ) = (left, right)
        else {
            return false;
        };
        self.push(result, other_result, depth);
        unsafe_ == other_unsafe
            && convention == other_convention
            && self.list(parameters, other_parameters, depth)
    }
    fn push(&mut self, x: TypeId, y: TypeId, depth: u8) {
        self.pending.push(Pair::Type(x, y, depth));
    }
    fn related(&mut self, x: GenericArgs, y: GenericArgs, depth: u8) -> bool {
        self.arguments(x, y, depth);
        true
    }
    fn constants(&self, x: ConstId, y: ConstId) -> Result<bool, &'static str> {
        if x == y {
            return Ok(true);
        }
        let kind = |id| {
            self.store
                .const_kind(id)
                .ok_or("invalid coherence header constant")
        };
        Ok(match (kind(x)?, kind(y)?) {
            (ConstKind::Value(p), ConstKind::Value(q)) => p == q,
            _ => true,
        })
    }
}
/// A rigid constructor with one type child.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Single {
    Reference(bool),
    Pointer(bool),
    Slice,
    /// Rustc compares a pattern type's base only.
    Pattern,
    Array(Length),
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Length {
    Value(u64),
    Term(ConstId),
}
impl Single {
    const fn of(kind: Kind) -> Option<(Self, TypeId)> {
        Some(match kind {
            Kind::Reference {
                mutable, referent, ..
            } => (Self::Reference(mutable), referent),
            Kind::Pointer { mutable, pointee } => (Self::Pointer(mutable), pointee),
            Kind::Slice(element) => (Self::Slice, element),
            Kind::Pattern { base, .. } => (Self::Pattern, base),
            Kind::Array { element, length } => (Self::Array(Length::Value(length)), element),
            Kind::ArrayTerm { element, length } => (Self::Array(Length::Term(length)), element),
            _ => return None,
        })
    }
}
/// Parameters, projections, opaque types and futures may unify with anything.
const fn open(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..) | Kind::Future { .. }
    )
}
