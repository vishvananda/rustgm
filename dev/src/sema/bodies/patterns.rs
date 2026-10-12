//! Pattern constraints retain selected fields and share resolved binding identities.
use super::{Checker, Constructor, Domain, Error, Kind, List, NodeId, TermId};
use crate::{
    sema::{
        members::{FieldId, Selection, VariantId},
        patterns::{Alternatives, Field, Fields, Kind as PatternKind, PatternId},
    },
    syntax::{ast::PatternFlavor, item::Shape},
};

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
    fn reserve_pattern(&mut self, mut node: NodeId, term: TermId) -> Result<PatternId, Error> {
        while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } = self.kind(node) {
            self.save(node, term);
            node = inner;
        }
        let id = self
            .facts
            .patterns
            .reserve(self.key(node))
            .map_err(|e| self.error(node, e))?;
        self.pattern_work.push((node, term, id));
        self.pattern_types.push((id, term));
        Ok(id)
    }
    pub(super) fn pattern(&mut self, root: NodeId, term: TermId) -> Result<(), Error> {
        let id = self.reserve_pattern(root, term)?;
        self.facts.patterns.roots.insert(self.key(root), id);
        let mut steps = 0;
        while let Some((node, term, id)) = self.pattern_work.pop() {
            steps += 1;
            if steps > 1_000_000 {
                return Err(self.error(root, "pattern type work budget exceeded"));
            }
            self.save(node, term);
            let mark = self.pattern_work.len();
            let kind = self.pattern_kind(node, term)?;
            self.pattern_work[mark..].reverse();
            self.facts.patterns.nodes[id.0 as usize].kind = kind;
        }
        Ok(())
    }
    fn pattern_kind(&mut self, node: NodeId, term: TermId) -> Result<PatternKind, Error> {
        match self.kind(node) {
            Kind::Wildcard => Ok(PatternKind::Wildcard),
            Kind::Binding {
                by_ref: false,
                mutable,
                subpattern,
                ..
            } if self
                .facts
                .node(self.key(node))
                .is_some_and(|fact| fact.binding.is_some()) =>
            {
                let definition = self
                    .facts
                    .node(self.key(node))
                    .and_then(|fact| fact.binding)
                    .ok_or_else(|| self.error(node, "missing pattern binding identity"))?;
                if let Some(prior) = self.locals[definition.0 as usize] {
                    self.equate(node, prior, term)?;
                } else {
                    self.locals[definition.0 as usize] = Some(term);
                    self.bindings.push(definition);
                }
                let subpattern = subpattern
                    .map(|child| self.reserve_pattern(child, term))
                    .transpose()?;
                Ok(PatternKind::Binding {
                    definition,
                    mutable,
                    subpattern,
                })
            }
            Kind::Binding { by_ref: true, .. } | Kind::ReferencePattern { .. } => {
                Err(self.error(node, "reference patterns require pattern borrow semantics"))
            }
            Kind::Pattern {
                flavor: PatternFlavor::Tuple,
                elements,
                ..
            } => self.positional_pattern(node, term, elements, None),
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
                self.positional_pattern(node, term, elements, Some(variant))
            }
            Kind::StructPattern { path, fields, rest } => {
                self.record_pattern(node, term, path, fields, rest)
            }
            Kind::Pattern {
                flavor: PatternFlavor::Or,
                elements,
                ..
            } => {
                let start = self.pattern_index(node, self.facts.patterns.alternatives.len())?;
                let unit = self.report.package.unit(self.file);
                for child in unit.configured.ast.list(elements) {
                    let id = self.reserve_pattern(*child, term)?;
                    self.facts.patterns.alternatives.push(id);
                }
                let end = self.pattern_index(node, self.facts.patterns.alternatives.len())?;
                if start == end {
                    return Err(self.error(node, "empty alternative pattern"));
                }
                Ok(PatternKind::Or(Alternatives { start, end }))
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
            return Err(self.error(
                node,
                "refutable constructor requires conditional pattern semantics",
            ));
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
            let pattern = self.reserve_pattern(child, child_term)?;
            self.facts
                .patterns
                .fields
                .push(Field { selection, pattern });
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
            let pattern = self.reserve_pattern(pattern, child_term)?;
            self.facts.patterns.fields.push(Field {
                selection: Selection::Nominal(field),
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
}
