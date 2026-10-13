//! Pattern constraints retain selected fields and share resolved binding identities.
use super::{Checker, Constructor, Domain, Error, Kind, List, NodeId, TermId};
use crate::{
    sema::{
        members::{FieldId, Selection, VariantId},
        model::{BodyRegion, BodyRegionOrigin},
        patterns::{
            Alternatives, Capture, Field, Fields, Kind as PatternKind, PatternId, Projection,
        },
    },
    syntax::{ast::PatternFlavor, item::Shape},
    types::RegionId,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Value,
    Reference { mutable: bool },
}
pub(super) struct Work {
    node: NodeId,
    term: TermId,
    id: PatternId,
    mode: Mode,
    rest: bool,
}
impl Checker<'_> {
    pub(super) fn simple_pattern(&self, mut node: NodeId) -> bool {
        while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } = self.kind(node) {
            node = inner;
        }
        matches!(self.kind(node), Kind::Wildcard)
            || matches!(
                self.kind(node),
                Kind::Binding {
                    by_ref: false,
                    subpattern: None,
                    ..
                }
            ) && self
                .facts
                .node(self.key(node))
                .is_some_and(|fact| fact.binding.is_some())
    }
    fn reserve_pattern(
        &mut self,
        node: NodeId,
        term: TermId,
        mode: Mode,
    ) -> Result<PatternId, Error> {
        self.schedule_pattern(node, term, mode, false)
    }
    fn schedule_pattern(
        &mut self,
        mut node: NodeId,
        term: TermId,
        mode: Mode,
        rest: bool,
    ) -> Result<PatternId, Error> {
        while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } = self.kind(node) {
            self.save(node, term);
            node = inner;
        }
        let id = self
            .facts
            .patterns
            .reserve(self.key(node))
            .map_err(|e| self.error(node, e))?;
        self.pattern_work.push(Work {
            node,
            term,
            id,
            mode,
            rest,
        });
        self.pattern_types.push((id, term));
        Ok(id)
    }
    pub(super) fn pattern(&mut self, root: NodeId, term: TermId) -> Result<(), Error> {
        let mark = self.range_work.len();
        let root = self.selected_pattern(root, term)?;
        if self.range_work.len() != mark || self.pattern_refutable {
            self.pattern_requirements.push(root);
        }
        Ok(())
    }
    pub(super) fn selected_pattern(
        &mut self,
        root: NodeId,
        term: TermId,
    ) -> Result<PatternId, Error> {
        self.pattern_refutable = false;
        let id = self.reserve_pattern(root, term, Mode::Value)?;
        self.facts.patterns.roots.insert(self.key(root), id);
        let mut steps = 0;
        while let Some(Work {
            node,
            term,
            id,
            mode,
            rest,
        }) = self.pattern_work.pop()
        {
            steps += 1;
            if steps > 1_000_000 {
                return Err(self.error(root, "pattern type work budget exceeded"));
            }
            self.save(node, term);
            let mark = self.pattern_work.len();
            let (id, term, mode) = self.pattern_adjust(node, term, id, mode, &mut steps)?;
            let kind = self.pattern_kind(node, term, id, mode, rest)?;
            self.pattern_work[mark..].reverse();
            self.facts.patterns.nodes[id.0 as usize].kind = kind;
        }
        Ok(id)
    }
    fn pattern_kind(
        &mut self,
        node: NodeId,
        term: TermId,
        id: PatternId,
        mode: Mode,
        rest: bool,
    ) -> Result<PatternKind, Error> {
        match self.kind(node) {
            Kind::Literal(_)
            | Kind::Unary {
                op: crate::syntax::ast::Unary::Negate,
                ..
            }
            | Kind::PatternRange { .. } => self.scalar_pattern(node, term),
            Kind::Wildcard => Ok(PatternKind::Wildcard),
            Kind::RestPattern if rest => Ok(PatternKind::Wildcard),
            Kind::Binding {
                by_ref,
                mutable,
                subpattern,
                ..
            } if self
                .facts
                .node(self.key(node))
                .is_some_and(|fact| fact.binding.is_some()) =>
            {
                self.pattern_binding(node, term, mode, (by_ref, mutable), subpattern, rest)
            }
            Kind::ReferencePattern { mutable, pattern } => {
                self.reference_pattern(node, term, mode, mutable, pattern)
            }
            Kind::Pattern {
                flavor: PatternFlavor::Slice,
                elements,
                ..
            } => self.array_pattern(node, term, elements, mode),
            Kind::Pattern {
                flavor: PatternFlavor::Tuple,
                elements,
                ..
            } => self.positional_pattern(node, term, elements, None, mode),
            Kind::Pattern {
                flavor: PatternFlavor::Variant,
                path: Some(path),
                elements,
            } => {
                let variant = self.pattern_constructor(node, path, term)?;
                if self.facts.members.variants[variant.0 as usize].shape != Shape::Tuple {
                    return Err(
                        self.error(node, "tuple-struct pattern requires a tuple constructor")
                    );
                }
                self.positional_pattern(node, term, elements, Some(variant), mode)
            }
            Kind::StructPattern { path, fields, rest } => {
                self.record_pattern(node, term, path, fields, rest, mode)
            }
            Kind::Pattern {
                flavor: PatternFlavor::Or,
                elements,
                ..
            } => {
                let start = self.pattern_index(node, self.facts.patterns.alternatives.len())?;
                let unit = self.report.package.unit(self.file);
                for child in unit.configured.ast.list(elements) {
                    let id = self.reserve_pattern(*child, term, mode)?;
                    self.facts.patterns.alternatives.push(id);
                }
                let end = self.pattern_index(node, self.facts.patterns.alternatives.len())?;
                if start == end {
                    return Err(self.error(node, "empty alternative pattern"));
                }
                Ok(PatternKind::Or(Alternatives { start, end }))
            }
            Kind::Binding { .. } | Kind::Path { .. } if self.pattern_constant(node).is_some() => {
                self.constant_pattern(node, term, id)
            }
            Kind::Binding { .. } | Kind::Path { .. } => {
                let variant = self.pattern_constructor(node, node, term)?;
                if self.facts.members.variants[variant.0 as usize].shape != Shape::Unit {
                    return Err(
                        self.error(node, "bare constructor pattern requires a unit variant")
                    );
                }
                let start = self.pattern_index(node, self.facts.patterns.fields.len())?;
                Ok(PatternKind::Product {
                    variant: Some(variant),
                    fields: Fields { start, end: start },
                })
            }
            _ => Err(self.error(
                node,
                "pattern requires conditional or reference pattern semantics",
            )),
        }
    }
    fn pattern_index(&self, node: NodeId, value: usize) -> Result<u32, Error> {
        u32::try_from(value).map_err(|_| self.error(node, "pattern identity space exhausted"))
    }
    fn pattern_region(
        &mut self,
        node: NodeId,
        origin: BodyRegionOrigin,
    ) -> Result<RegionId, Error> {
        let ordinal = u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .filter(|n| *n < 0x8000_0000)
            .ok_or_else(|| self.error(node, "body region identity space exhausted"))?;
        self.facts.body_regions.push(BodyRegion {
            body: self.body,
            source: self.key(node),
            origin,
        });
        Ok(RegionId(ordinal))
    }
    fn pattern_adjust(
        &mut self,
        node: NodeId,
        mut term: TermId,
        mut id: PatternId,
        mut mode: Mode,
        steps: &mut usize,
    ) -> Result<(PatternId, TermId, Mode), Error> {
        let structural = matches!(
            self.kind(node),
            Kind::Literal(_)
                | Kind::Unary {
                    op: crate::syntax::ast::Unary::Negate,
                    ..
                }
                | Kind::PatternRange { .. }
                | Kind::Pattern {
                    flavor: PatternFlavor::Slice | PatternFlavor::Tuple | PatternFlavor::Variant,
                    ..
                }
                | Kind::StructPattern { .. }
                | Kind::Path { .. }
        ) || matches!(self.kind(node), Kind::Binding { .. })
            && self
                .facts
                .node(self.key(node))
                .is_some_and(|fact| fact.binding.is_none());
        let reference_constant = self
            .pattern_constant(node)
            .and_then(|definition| self.facts.definitions[definition.0 as usize])
            .is_some_and(|ty| {
                matches!(
                    self.facts.types.kind(ty),
                    Some(crate::types::Kind::Reference { .. })
                )
            });
        if structural && !reference_constant {
            while let Some(Constructor::Reference { mutable, .. }) = self.head(node, term)? {
                *steps += 1;
                if *steps > 1_000_000 {
                    return Err(self.error(node, "pattern type work budget exceeded"));
                }
                mode = Mode::Reference {
                    mutable: mutable && mode != Mode::Reference { mutable: false },
                };
                term = self.child(node, term, 0)?;
                let child = self
                    .facts
                    .patterns
                    .reserve(self.key(node))
                    .map_err(|e| self.error(node, e))?;
                self.pattern_types.push((child, term));
                self.facts.patterns.nodes[id.0 as usize].kind = PatternKind::Dereference {
                    mutable,
                    implicit: true,
                    pattern: child,
                };
                id = child;
            }
        }
        Ok((id, term, mode))
    }
    fn pattern_binding(
        &mut self,
        node: NodeId,
        term: TermId,
        mode: Mode,
        written: (bool, bool),
        subpattern: Option<NodeId>,
        rest: bool,
    ) -> Result<PatternKind, Error> {
        let (by_ref, mutable) = written;
        if mode != Mode::Value && (by_ref || mutable) {
            return Err(self.error(
                node,
                "explicit binding qualifiers require the move default binding mode in edition 2024",
            ));
        }
        let capture_mode = if by_ref {
            Mode::Reference { mutable }
        } else {
            mode
        };
        let (capture, local) = match capture_mode {
            Mode::Value => (Capture::Value, term),
            Mode::Reference { mutable } => {
                let region = self.pattern_region(node, BodyRegionOrigin::PatternBorrow)?;
                let local = self
                    .infer
                    .construct(Constructor::Reference { mutable, region }, &[term])
                    .map_err(|e| self.error(node, e))?;
                (Capture::Borrow { mutable, region }, local)
            }
        };
        let definition = self
            .facts
            .node(self.key(node))
            .and_then(|fact| fact.binding)
            .ok_or_else(|| self.error(node, "missing pattern binding identity"))?;
        if let Some(prior) = self.locals[definition.0 as usize] {
            self.equate(node, prior, local)?;
        } else {
            self.locals[definition.0 as usize] = Some(local);
            self.bindings.push(definition);
        }
        let subpattern = subpattern
            .map(|child| self.schedule_pattern(child, term, mode, rest))
            .transpose()?;
        Ok(PatternKind::Binding {
            definition,
            mutable: mutable && !by_ref,
            capture,
            subpattern,
        })
    }
    fn reference_pattern(
        &mut self,
        node: NodeId,
        term: TermId,
        mode: Mode,
        mutable: bool,
        pattern: NodeId,
    ) -> Result<PatternKind, Error> {
        if mode != Mode::Value {
            return Err(self.error(
                node,
                "reference patterns require the move default binding mode in edition 2024",
            ));
        }
        if self.head(node, term)?.is_none() {
            let region = self.pattern_region(node, BodyRegionOrigin::PatternInput)?;
            let referent = self.variable(node, Domain::Any)?;
            let reference = self
                .infer
                .construct(Constructor::Reference { mutable, region }, &[referent])
                .map_err(|e| self.error(node, e))?;
            self.equate(node, term, reference)?;
        }
        let Some(Constructor::Reference {
            mutable: actual, ..
        }) = self.head(node, term)?
        else {
            return Err(self.error(node, "reference pattern requires a reference type"));
        };
        if mutable != actual {
            return Err(self.error(node, "reference pattern mutability disagrees with its type"));
        }
        let child = self.child(node, term, 0)?;
        let pattern = self.reserve_pattern(pattern, child, Mode::Value)?;
        Ok(PatternKind::Dereference {
            mutable,
            implicit: false,
            pattern,
        })
    }
    fn slice_pattern(
        &mut self,
        node: NodeId,
        term: TermId,
        elements: &[NodeId],
        rest: Option<usize>,
        mode: Mode,
    ) -> Result<PatternKind, Error> {
        let minimum = u64::try_from(elements.len() - usize::from(rest.is_some()))
            .map_err(|_| self.error(node, "slice pattern arity overflow"))?;
        self.pattern_refutable |= rest.is_none() || minimum != 0;
        let element = self.child(node, term, 0)?;
        let start = self.pattern_index(node, self.facts.patterns.fields.len())?;
        for (index, &child) in elements.iter().enumerate() {
            let ordinal = u64::try_from(index)
                .map_err(|_| self.error(child, "slice pattern index overflow"))?;
            let is_rest = rest == Some(index);
            let (selection, child_term) = if is_rest {
                if matches!(self.kind(child), Kind::RestPattern) {
                    self.save(child, term);
                    continue;
                }
                (
                    Projection::Subslice {
                        prefix: ordinal,
                        suffix: minimum - ordinal,
                    },
                    term,
                )
            } else if rest.is_some_and(|rest| index > rest) {
                (Projection::FromEnd(minimum - ordinal + 1), element)
            } else {
                (Projection::Index(ordinal), element)
            };
            let pattern = self.schedule_pattern(child, child_term, mode, is_rest)?;
            self.facts
                .patterns
                .fields
                .push(Field { selection, pattern });
        }
        let end = self.pattern_index(node, self.facts.patterns.fields.len())?;
        Ok(PatternKind::Slice {
            minimum,
            exact: rest.is_none(),
            fields: Fields { start, end },
        })
    }
    fn pattern_constructor(
        &mut self,
        node: NodeId,
        path: NodeId,
        term: TermId,
    ) -> Result<VariantId, Error> {
        let id = self.constructor(path)?;
        let variant = self.facts.members.variants[id.0 as usize];
        let definition = self
            .facts
            .aggregates
            .get(variant.owner)
            .ok_or_else(|| self.error(node, "missing pattern constructor shape"))?;
        if definition.end - definition.start != 1 {
            self.pattern_refutable = true;
        }
        let ty = self.facts.definitions[variant.owner.0 as usize]
            .ok_or_else(|| self.error(node, "missing pattern constructor type"))?;
        let expected = self.known(node, ty)?;
        self.equate(node, term, expected)?;
        Ok(id)
    }
    fn positional_pattern(
        &mut self,
        node: NodeId,
        term: TermId,
        elements: List,
        variant: Option<VariantId>,
        mode: Mode,
    ) -> Result<PatternKind, Error> {
        let unit = self.report.package.unit(self.file);
        let elements = unit.configured.ast.list(elements);
        let rest = elements
            .iter()
            .position(|n| matches!(self.kind(*n), Kind::RestPattern));
        if rest.is_some()
            && elements
                .iter()
                .filter(|n| matches!(self.kind(**n), Kind::RestPattern))
                .count()
                != 1
        {
            return Err(self.error(node, "multiple rest patterns in one product"));
        }
        let explicit = elements.len() - usize::from(rest.is_some());
        let arity = if let Some(id) = variant {
            let variant = self.facts.members.variants[id.0 as usize];
            (variant.end - variant.start) as usize
        } else {
            if self.head(node, term)?.is_none() && rest.is_none() {
                self.type_children.clear();
                for _ in 0..explicit {
                    let child = self.variable(node, Domain::Any)?;
                    self.type_children.push(child);
                }
                let tuple = self
                    .infer
                    .construct(Constructor::Tuple, &self.type_children)
                    .map_err(|e| self.error(node, e))?;
                self.equate(node, term, tuple)?;
            }
            if self.head(node, term)? != Some(Constructor::Tuple) {
                return Err(self.error(node, "tuple pattern requires a known tuple type"));
            }
            self.infer
                .arity(&self.facts.types, term)
                .map_err(|e| self.error(node, e))?
                .ok_or_else(|| self.error(node, "missing tuple pattern arity"))?
        };
        if explicit > arity || (rest.is_none() && explicit != arity) {
            return Err(self.error(node, "pattern field count disagrees with its type"));
        }
        let start = self.pattern_index(node, self.facts.patterns.fields.len())?;
        for (index, &child) in elements.iter().enumerate() {
            if Some(index) == rest {
                self.save(child, term);
                continue;
            }
            let ordinal = if rest.is_some_and(|rest| index > rest) {
                arity - (elements.len() - index)
            } else {
                index
            };
            let (selection, child_term) = if let Some(id) = variant {
                let field = FieldId(
                    self.facts.members.variants[id.0 as usize].start
                        + self.pattern_index(node, ordinal)?,
                );
                self.accessible(child, field)?;
                (
                    Selection::Nominal(field),
                    self.known(child, self.facts.members.fields[field.0 as usize].ty)?,
                )
            } else {
                (
                    Selection::Tuple(self.pattern_index(node, ordinal)?),
                    self.child(node, term, ordinal)?,
                )
            };
            let pattern = self.reserve_pattern(child, child_term, mode)?;
            self.facts.patterns.fields.push(Field {
                selection: Projection::Field(selection),
                pattern,
            });
        }
        let end = self.pattern_index(node, self.facts.patterns.fields.len())?;
        Ok(PatternKind::Product {
            variant,
            fields: Fields { start, end },
        })
    }
    fn record_pattern(
        &mut self,
        node: NodeId,
        term: TermId,
        path: NodeId,
        fields: List,
        rest: bool,
        mode: Mode,
    ) -> Result<PatternKind, Error> {
        let variant_id = self.pattern_constructor(node, path, term)?;
        let variant = self.facts.members.variants[variant_id.0 as usize];
        self.field_seen.clear();
        self.field_seen
            .resize((variant.end - variant.start) as usize, false);
        let start = self.pattern_index(node, self.facts.patterns.fields.len())?;
        let unit = self.report.package.unit(self.file);
        for &wrapper in unit.configured.ast.list(fields) {
            if !self.active(wrapper) {
                continue;
            }
            let mut field_node = wrapper;
            while let Kind::Attributed { value, .. } = self.kind(field_node) {
                field_node = value;
            }
            let Kind::PatternField { name, pattern } = self.kind(field_node) else {
                return Err(self.error(field_node, "missing pattern field syntax"));
            };
            let name = self.member_name(field_node, name)?;
            let field = self
                .facts
                .members
                .field(variant_id, name)
                .ok_or_else(|| self.error(field_node, "unknown pattern field"))?;
            self.accessible(field_node, field)?;
            if std::mem::replace(
                &mut self.field_seen[(field.0 - variant.start) as usize],
                true,
            ) {
                return Err(self.error(field_node, "duplicate pattern field"));
            }
            let ty = self.facts.members.fields[field.0 as usize].ty;
            let child_term = self.known(field_node, ty)?;
            self.save(wrapper, child_term);
            self.save(field_node, child_term);
            let pattern = self.reserve_pattern(pattern, child_term, mode)?;
            self.facts.patterns.fields.push(Field {
                selection: Projection::Field(Selection::Nominal(field)),
                pattern,
            });
        }
        if !rest && self.field_seen.contains(&false) {
            return Err(self.error(node, "missing pattern fields without rest"));
        }
        let end = self.pattern_index(node, self.facts.patterns.fields.len())?;
        Ok(PatternKind::Product {
            variant: Some(variant_id),
            fields: Fields { start, end },
        })
    }
    fn slice_rest(&self, node: NodeId) -> bool {
        match self.kind(node) {
            Kind::RestPattern => true,
            Kind::Binding {
                subpattern: Some(child),
                ..
            } => {
                matches!(self.kind(child), Kind::RestPattern)
            }
            _ => false,
        }
    }
    fn array_pattern(
        &mut self,
        node: NodeId,
        term: TermId,
        elements: List,
        mode: Mode,
    ) -> Result<PatternKind, Error> {
        let unit = self.report.package.unit(self.file);
        let elements = unit.configured.ast.list(elements);
        let mut rest = None;
        for (index, &child) in elements.iter().enumerate() {
            if self.slice_rest(child) && rest.replace(index).is_some() {
                return Err(self.error(child, "multiple rest patterns in one array"));
            }
        }
        let explicit = u64::try_from(elements.len() - usize::from(rest.is_some()))
            .map_err(|_| self.error(node, "array pattern arity overflow"))?;
        if self.head(node, term)?.is_none() && rest.is_none() {
            let element = self.variable(node, Domain::Any)?;
            let array = self
                .infer
                .construct(Constructor::Array(explicit), &[element])
                .map_err(|e| self.error(node, e))?;
            self.equate(node, term, array)?;
        }
        if self.head(node, term)? == Some(Constructor::Slice) {
            return self.slice_pattern(node, term, elements, rest, mode);
        }
        let Some(Constructor::Array(length)) = self.head(node, term)? else {
            return Err(self.error(node, "array pattern requires a known fixed-array type"));
        };
        if explicit > length || (rest.is_none() && explicit != length) {
            return Err(self.error(node, "array pattern length disagrees with its type"));
        }
        let element = self.child(node, term, 0)?;
        let start = self.pattern_index(node, self.facts.patterns.fields.len())?;
        for (index, &child) in elements.iter().enumerate() {
            let ordinal = u64::try_from(index)
                .map_err(|_| self.error(child, "array pattern index overflow"))?;
            let is_rest = rest == Some(index);
            let (selection, child_term) = if is_rest {
                if matches!(self.kind(child), Kind::RestPattern) {
                    self.save(child, term);
                    continue;
                }
                let remaining = length - explicit;
                let ty = self
                    .infer
                    .construct(Constructor::Array(remaining), &[element])
                    .map_err(|e| self.error(child, e))?;
                (
                    Projection::Subarray {
                        start: ordinal,
                        end: ordinal + remaining,
                    },
                    ty,
                )
            } else {
                let index = if rest.is_some_and(|rest| index > rest) {
                    length - explicit + ordinal - 1
                } else {
                    ordinal
                };
                (Projection::Index(index), element)
            };
            let pattern = self.schedule_pattern(child, child_term, mode, is_rest)?;
            self.facts
                .patterns
                .fields
                .push(Field { selection, pattern });
        }
        let end = self.pattern_index(node, self.facts.patterns.fields.len())?;
        Ok(PatternKind::Product {
            variant: None,
            fields: Fields { start, end },
        })
    }
}
