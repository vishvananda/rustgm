//! Full semantic fields are resolved from owned roots; physical shape types grant no goal.
use super::{Form, Origin, Parts, Selected, Shape, Step, reserve};
use crate::{
    body::{defining_cleanups::Subject, ir::Body},
    support::{
        hash::{HashMap, HashSet},
        retained::Growth,
    },
    types::{
        Kind, Store, TypeId,
        aggregate::{Definitions, SemanticFields},
    },
};
pub(super) struct Resolver<'a> {
    fields: HashMap<TypeId, SemanticFields<'a>>,
    checked: HashSet<(TypeId, super::parts::Id)>,
}
impl Selected {
    pub(in crate::borrow::ownership::variant_liveness) fn resolve(
        &mut self,
        types: &mut Store,
        definitions: &Definitions,
        body: &Body,
        parts: &Parts,
        constructors: &[super::super::trans::Candidate],
        coordinates: Option<&[TypeId]>,
    ) -> Result<(), &'static str> {
        if self.resolved || self.body != body.key() || self.source != body.source {
            return Err(
                "selected cleanup cannot resolve another body or repeat its semantic stage",
            );
        }
        for root in &self.roots {
            parts.charge(1)?;
            if body.place_type(root.place) != Some(root.ty) {
                return Err("selected cleanup changed its full semantic root application");
            }
        }
        let resolved = if let Some(coordinates) = coordinates {
            if coordinates.len() != self.nodes.len() {
                return Err("selected cleanup changed its resolved coordinate inventory");
            }
            std::borrow::Cow::Borrowed(coordinates)
        } else {
            std::borrow::Cow::Owned(self.coordinates(
                types,
                definitions,
                body,
                parts,
                constructors,
            )?)
        };
        reserve(parts, &mut self.subjects, self.leaves.len())?;
        for row in &mut self.rows {
            let start = super::parts::compact(self.subjects.len())?;
            for leaf in self
                .leaves
                .get(row.leaves.start as usize..row.leaves.end as usize)
                .ok_or("selected cleanup leaves are missing")?
            {
                parts.charge(1)?;
                if row.cleanup.live_type.is_none() && !leaf.history {
                    continue;
                }
                let ty = match leaf.origin {
                    Origin::Direct(ty) => ty,
                    Origin::Coordinate(c) => *resolved
                        .get(c.0 as usize)
                        .ok_or("selected cleanup field is unresolved")?,
                };
                self.subjects.push(Subject {
                    ty,
                    purpose: leaf.purpose,
                    access: leaf.access && (leaf.history || row.cleanup.access),
                });
            }
            row.subjects = super::Range {
                start,
                end: super::parts::compact(self.subjects.len())?,
            };
        }
        self.resolved = true;
        Ok(())
    }
    pub(in crate::borrow::ownership::variant_liveness) fn coordinates(
        &self,
        types: &mut Store,
        definitions: &Definitions,
        body: &Body,
        parts: &Parts,
        constructors: &[super::super::trans::Candidate],
    ) -> Result<Vec<TypeId>, &'static str> {
        let mut resolved = Vec::new();
        let mut memo = Resolver::new();
        memo.constructors(types, definitions, parts, body, constructors)?;
        reserve(parts, &mut resolved, self.nodes.len())?;
        for (index, node) in self.nodes.iter().enumerate() {
            parts.charge(1)?;
            let ty = if let Some(parent) = node.parent {
                self.nodes
                    .get(parent.0 as usize)
                    .filter(|p| p.root == node.root && (parent.0 as usize) < index)
                    .ok_or("selected cleanup changed its semantic parent coordinate")?;
                let parent_ty = *resolved
                    .get(parent.0 as usize)
                    .ok_or("selected cleanup parent type is unresolved")?;
                memo.field(types, definitions, parts, parent_ty, node.step)?
            } else {
                if !matches!(node.step, Step::Root) {
                    return Err("selected cleanup has a non-root semantic origin");
                }
                self.roots
                    .get(node.root as usize)
                    .ok_or("selected cleanup root is missing")?
                    .ty
            };
            let shape = self
                .shapes
                .get(node.shape.0 as usize)
                .ok_or("selected cleanup shape is missing")?;
            memo.authenticate(
                types,
                definitions,
                parts,
                ty,
                (node.shape, shape),
                &self.counts,
            )?;
            resolved.push(ty);
        }
        Ok(resolved)
    }
}
impl<'a> Resolver<'a> {
    pub(super) fn new() -> Self {
        Self {
            fields: HashMap::default(),
            checked: HashSet::default(),
        }
    }
    pub(super) fn field(
        &mut self,
        types: &mut Store,
        definitions: &'a Definitions,
        parts: &Parts,
        parent: TypeId,
        step: Step,
    ) -> Result<TypeId, &'static str> {
        match (types.kind(parent), step) {
            (Some(Kind::Tuple(list)), Step::Tuple(index)) => types
                .list(list)
                .and_then(|v| v.get(index as usize))
                .copied()
                .ok_or("selected cleanup tuple field is missing"),
            (Some(Kind::Closure { environment, .. }), Step::Tuple(index)) => {
                let Some(Kind::Tuple(list)) = types.kind(environment) else {
                    return Err("selected cleanup closure environment changed");
                };
                types
                    .list(list)
                    .and_then(|v| v.get(index as usize))
                    .copied()
                    .ok_or("selected cleanup capture field is missing")
            }
            (
                Some(Kind::Array { element, .. } | Kind::ArrayTerm { element, .. }),
                Step::Element,
            ) => Ok(element),
            (Some(Kind::Adt(_, _)), Step::Nominal { variant, field }) => {
                if !self.fields.contains_key(&parent) {
                    parts.pay(Growth::map::<TypeId, SemanticFields<'_>>(
                        self.fields.len(),
                        self.fields.capacity(),
                        1,
                    ))?;
                    self.fields
                        .try_reserve(1)
                        .map_err(|_| "selected semantic fold allocation failed")?;
                    let fields = definitions.semantic_fields(types, parent)?;
                    parts.pay(Growth {
                        bytes: fields.retained_bytes(),
                        work: fields.work(),
                    })?;
                    self.fields.insert(parent, fields);
                }
                let fields = self
                    .fields
                    .get_mut(&parent)
                    .ok_or("selected semantic field fold is missing")?;
                let (work, bytes) = (fields.work(), fields.retained_bytes());
                let result = fields.field(types, variant as usize, field as usize);
                let paid = parts.pay(Growth {
                    work: fields.work().saturating_sub(work),
                    bytes: fields.retained_bytes().saturating_sub(bytes),
                });
                paid?;
                result
            }
            _ => Err("selected cleanup field changed its actual semantic owner"),
        }
    }
    fn authenticate(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        parts: &Parts,
        ty: TypeId,
        (id, shape): (super::parts::Id, &Shape),
        counts: &[u32],
    ) -> Result<(), &'static str> {
        parts.charge(1)?;
        if self.checked.contains(&(ty, id)) {
            return Ok(());
        }
        let expected = counts
            .get(shape.counts.start as usize..shape.counts.end as usize)
            .ok_or("selected declaration dimensions are missing")?;
        match (shape.form, types.kind(ty), types.kind(shape.ty)) {
            (
                Form::Nominal { enumeration },
                Some(Kind::Adt(owner, args)),
                Some(Kind::Adt(geometry, _)),
            ) if owner == geometry => {
                types
                    .arguments(args)
                    .ok_or("selected full nominal arguments are missing")?;
                let definition = definitions
                    .get(owner)
                    .filter(|d| (d.kind == crate::types::aggregate::Kind::Enum) == enumeration)
                    .ok_or("selected nominal kind changed")?;
                let variants = definitions.variants(definition);
                parts.charge(variants.len())?;
                if variants.len() != expected.len() {
                    return Err("selected nominal variant inventory changed");
                }
                for (variant, &count) in variants.iter().zip(expected) {
                    if types
                        .list(variant.fields)
                        .is_none_or(|v| v.len() != count as usize)
                    {
                        return Err("selected nominal field inventory changed");
                    }
                }
            }
            (Form::Tuple, Some(kind), _) => {
                let list = match kind {
                    Kind::Tuple(list) => list,
                    Kind::Closure { environment, .. } => match types.kind(environment) {
                        Some(Kind::Tuple(list)) => list,
                        _ => return Err("selected closure environment is not a tuple"),
                    },
                    _ => return Err("selected tuple owner changed"),
                };
                if expected.len() != 1
                    || types
                        .list(list)
                        .is_none_or(|v| v.len() != expected[0] as usize)
                {
                    return Err("selected tuple field inventory changed");
                }
            }
            (Form::Array, Some(Kind::Array { .. } | Kind::ArrayTerm { .. }), _)
                if expected == [1] => {}
            (Form::Leaf, Some(Kind::Adt(owner, args)), Some(Kind::Adt(geometry, _)))
                if owner == geometry =>
            {
                types
                    .arguments(args)
                    .ok_or("selected full owner arguments are missing")?;
            }
            (Form::Leaf, Some(actual), Some(geometry))
                if leaf_identity(types, actual, geometry) => {}
            _ => return Err("selected cleanup changed its actual structural kind"),
        }
        parts.pay(Growth::map::<(TypeId, super::parts::Id), ()>(
            self.checked.len(),
            self.checked.capacity(),
            1,
        ))?;
        self.checked
            .try_reserve(1)
            .map_err(|_| "selected shape authentication allocation failed")?;
        self.checked.insert((ty, id));
        Ok(())
    }
}

fn leaf_identity(types: &Store, actual: Kind, geometry: Kind) -> bool {
    match (actual, geometry) {
        (Kind::Parameter(a), Kind::Parameter(b)) => a == b,
        (Kind::Projection(a), Kind::Projection(b)) => {
            a.item == b.item
                && matches!((types.trait_application(a.trait_application), types.trait_application(b.trait_application)), (Some(a), Some(b)) if a.item == b.item)
        }
        (Kind::Opaque(a, _), Kind::Opaque(b, _))
        | (Kind::Future { identity: a, .. }, Kind::Future { identity: b, .. }) => a == b,
        (
            Kind::Dynamic {
                shape: a,
                bound_regions: ar,
                ..
            },
            Kind::Dynamic {
                shape: b,
                bound_regions: br,
                ..
            },
        ) => a == b && ar == br,
        (Kind::Foreign(a), Kind::Foreign(b)) => a == b,
        // Nominal applications are checked above against their actual owner.
        (Kind::Adt(..), _) | (_, Kind::Adt(..)) => false,
        (a, b) => std::mem::discriminant(&a) == std::mem::discriminant(&b),
    }
}
