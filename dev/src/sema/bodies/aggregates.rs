//! Aggregate constraints publish member selections once for bodies and ownership.
use super::{
    Checker, Constructor, DefId, DefKind, Domain, Error, Kind, List, NodeId, Operation, Primitive,
    Target, TermId, Type, TypeId, Work, declarations,
};
use crate::{
    resolve::model::Access,
    sema::members::{FieldId, Name, Selection, VariantId},
    syntax::{item::Shape, token_tree::Span},
    types::aggregate,
};

#[derive(Clone, Copy)]
pub(super) struct Field {
    receiver: TermId,
    name: Name,
    kind: Constructor,
}

impl Checker<'_> {
    pub(super) fn selected(&mut self, node: NodeId, operation: Operation) -> Result<(), Error> {
        let key = self.key(node);
        self.facts
            .node_mut(key)
            .ok_or_else(|| {
                super::Declarations::error(self.report, key, "missing aggregate expression fact")
            })?
            .operation = Some(operation);
        Ok(())
    }
    fn shape(
        &mut self,
        node: NodeId,
        constructor: Constructor,
        expected: Option<TermId>,
    ) -> Result<TermId, Error> {
        let term = self
            .state
            .infer
            .construct(constructor, &self.state.type_children)
            .map_err(|e| self.error(node, e))?;
        if let Some(expected) = expected {
            // Borrowing can supply a slice element type before reference unsizing.
            if matches!(constructor, Constructor::Array(_))
                && self.contextual_head(node, expected)? == Some(Constructor::Slice)
            {
                let element = self.child(node, expected, 0)?;
                self.shape_equation(node, self.state.type_children[0], element)?;
            } else {
                self.shape_equation(node, term, expected)?;
            }
        }
        self.save(node, term);
        Ok(term)
    }
    pub(super) fn enter_elements(
        &mut self,
        node: NodeId,
        elements: List,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let tuple = matches!(self.kind(node), Kind::Tuple(_));
        let unit = self.report.package.unit(self.state.file);
        let count = unit
            .configured
            .ast
            .list(elements)
            .iter()
            .filter(|n| self.active(**n))
            .count();
        self.state.type_children.clear();
        for _ in 0..if tuple { count } else { 1 } {
            let child = self.variable(node, Domain::Any)?;
            self.state.type_children.push(child);
        }
        let constructor = if tuple {
            Constructor::Tuple
        } else {
            Constructor::Array(
                u64::try_from(count).map_err(|_| self.error(node, "too many array elements"))?,
            )
        };
        let term = self.shape(node, constructor, expected)?;
        let mut ordinal = count;
        for child in unit.configured.ast.list(elements).iter().rev().copied() {
            if !self.active(child) {
                continue;
            }
            ordinal -= 1;
            let expected = self.child(node, term, if tuple { ordinal } else { 0 })?;
            if tuple {
                // Later fields observe the context inferred from earlier ones,
                // including whether an existing mutable reference is reborrowed.
                self.state.work.push(Work::CoerceElement(child, expected));
            }
            self.state.work.push(Work::Enter(child, Some(expected)));
        }
        Ok(())
    }
    pub(super) fn finish_elements(
        &mut self,
        node: NodeId,
        elements: List,
    ) -> Result<TermId, Error> {
        let term = self.term(node)?;
        if matches!(self.kind(node), Kind::Tuple(_)) {
            return Ok(term);
        }
        let unit = self.report.package.unit(self.state.file);
        let element = self.child(node, term, 0)?;
        if self.head(node, element)?.is_none() {
            let mut common = None;
            for child in unit.configured.ast.list(elements) {
                if !self.active(*child) {
                    continue;
                }
                let current = self.term(*child)?;
                common = Some(if let Some(prior) = common {
                    self.join_types(node, prior, current)?
                } else {
                    current
                });
            }
            if let Some(common) = common {
                self.equate(node, element, common)?;
            }
        }
        let mut reference_hint = false;
        for child in unit.configured.ast.list(elements) {
            if !self.active(*child) {
                continue;
            }
            if reference_hint {
                self.state.reference_expected.insert(*child);
            }
            reference_hint |= matches!(
                self.head(*child, self.term(*child)?)?,
                Some(Constructor::Reference { .. })
            );
            self.coerce(*child, element)?;
        }
        Ok(term)
    }
    pub(super) fn enter_repeat(
        &mut self,
        node: NodeId,
        value: NodeId,
        count: NodeId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let constructor = if let Some(length) =
            self.facts.const_parameter(self.report, self.key(count))
        {
            if self.facts.types.const_type(length) != Some(TypeId::primitive(Primitive::Usize)) {
                return Err(self.error(count, "array length must be usize"));
            }
            Constructor::ArrayTerm(length)
        } else {
            Constructor::Array(declarations::array_length(self.report, self.key(count))?)
        };
        let element = self.variable(node, Domain::Any)?;
        self.state.type_children.clear();
        self.state.type_children.push(element);
        let term = self.shape(node, constructor, expected)?;
        let expected = self.child(node, term, 0)?;
        self.state.work.push(Work::Enter(value, Some(expected)));
        Ok(())
    }
    pub(super) fn finish_repeat(&mut self, node: NodeId, value: NodeId) -> Result<TermId, Error> {
        let term = self.term(node)?;
        let element = self.child(node, term, 0)?;
        let mark = self.state.adjustments.len();
        self.coerce(value, element)?;
        if self.head(node, term)? == Some(Constructor::Array(0)) {
            // No element is stored. Keep higher-ranked type compatibility while
            // removing the discarded value's free storage-lifetime obligations.
            for index in mark..self.state.adjustments.len() {
                let original = self.state.adjustments[index].actual;
                let target = self.state.adjustments[index].expected;
                let actual = self.inferred_regions(value, original)?;
                let expected = self.inferred_regions(value, target)?;
                if actual != original || expected != target {
                    self.state
                        .discarded_adjustments
                        .insert(index, (actual, expected));
                }
            }
        }
        Ok(term)
    }
    pub(super) fn validate_repeat(&mut self, node: NodeId, mut value: NodeId) -> Result<(), Error> {
        let (element, repeats) = match self.facts.types.kind(self.ty(node)?) {
            Some(Type::Array { element, length }) => (element, length > 1),
            Some(Type::ArrayTerm { element, .. }) => (element, true),
            _ => return Err(self.error(node, "repeat has no solved array type")),
        };
        while let Kind::Group(child) = self.kind(value) {
            value = child;
        }
        let constant = self
            .facts
            .node(self.key(value))
            .and_then(|f| f.reference)
            .is_some_and(|reference| {
                matches!(reference, Target::Definition(id)
                if self.report.resolution.definitions[id.0 as usize].kind == super::DefKind::Const)
            });
        if constant {
            self.facts.constant_repeats.insert(self.key(node));
        }
        if repeats
            && !constant
            && !self
                .state
                .parameter_copies
                .as_mut()
                .unwrap_or(&mut self.facts.copies)
                .demand(
                    &self.facts.types,
                    &self.facts.aggregates,
                    &self.facts.derives,
                    element,
                )
                .map_err(|e| self.error(node, e))?
                .copy
        {
            return Err(self.error(node, "repeated array element must implement Copy"));
        }
        Ok(())
    }
    pub(super) fn select_unit_constructor(
        &mut self,
        node: NodeId,
        definition: DefId,
    ) -> Result<(), Error> {
        let Some(variant) = self.facts.members.constructors.get(&definition).copied() else {
            return Ok(());
        };
        if self.facts.members.variants[variant.0 as usize].shape == Shape::Unit {
            self.selected(node, Operation::Construct { variant })?;
        }
        Ok(())
    }
    pub(super) fn constructor(&self, node: NodeId) -> Result<VariantId, Error> {
        let Some(Target::Definition(definition)) =
            self.facts.node(self.key(node)).and_then(|f| f.reference)
        else {
            return Err(self.error(node, "aggregate path requires a resolved constructor"));
        };
        if let Some(variant) = self.facts.members.constructors.get(&definition) {
            return Ok(*variant);
        }
        if let Some(Type::Adt(item, _)) =
            self.facts.definitions[definition.0 as usize].and_then(|ty| self.facts.types.kind(ty))
        {
            return self.struct_variant(node, item);
        }
        Err(self.error(node, "path does not name a struct or enum variant"))
    }
    fn struct_variant(&self, node: NodeId, item: crate::types::ItemId) -> Result<VariantId, Error> {
        let definition = &self.report.resolution.definitions[item.0 as usize];
        if definition.kind != DefKind::Struct {
            return Err(self.error(node, "enum fields require a selected variant"));
        }
        definition
            .node
            .and_then(|key| self.facts.members.at_node(key))
            .ok_or_else(|| self.error(node, "missing struct member facts"))
    }
    pub(super) fn member_name(&mut self, node: NodeId, span: Span) -> Result<Name, Error> {
        let spelling = self
            .report
            .package
            .unit(self.state.file)
            .spelling(span)
            .map_err(|e| self.error(node, e.message))?;
        if spelling.as_bytes().first().is_some_and(u8::is_ascii_digit) {
            if !spelling.bytes().all(|b| b.is_ascii_digit())
                || (spelling.len() > 1 && spelling.starts_with('0'))
            {
                return Err(self.error(node, "tuple field must be an unsuffixed decimal index"));
            }
            return spelling
                .parse::<u32>()
                .map(Name::Index)
                .map_err(|_| self.error(node, "tuple field index overflows"));
        }
        self.facts
            .members
            .names
            .intern(spelling)
            .map(Name::Named)
            .map_err(|e| self.error(node, e))
    }
    pub(super) fn accessible(&self, node: NodeId, field: FieldId) -> Result<(), Error> {
        let access = self.facts.members.fields[field.0 as usize].access;
        if let Access::Within(module) = access
            && !self.report.package.is_descendant(self.state.module, module)
        {
            return Err(self.error(node, "aggregate field is private"));
        }
        Ok(())
    }
    pub(super) fn enter_struct(
        &mut self,
        node: NodeId,
        path: NodeId,
        fields: List,
        base: Option<NodeId>,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let variant_id = self.constructor(path)?;
        let variant = self.facts.members.variants[variant_id.0 as usize];
        let term = self.nominal_term(node, path, variant.owner, expected)?;
        if let Some(expected) = expected {
            self.shape_equation(node, term, expected)?;
        }
        self.save(node, term);
        self.selected(
            node,
            Operation::Construct {
                variant: variant_id,
            },
        )?;
        self.state.field_seen.clear();
        self.state
            .field_seen
            .resize((variant.end - variant.start) as usize, false);
        if let Some(base) = base {
            let definition = self
                .facts
                .aggregates
                .get(variant.owner)
                .ok_or_else(|| self.error(node, "missing aggregate shape"))?;
            if definition.kind != aggregate::Kind::Struct {
                return Err(self.error(node, "functional update requires a struct"));
            }
            for index in variant.start..variant.end {
                self.accessible(node, FieldId(index))?;
            }
            self.state.work.push(Work::Enter(base, Some(term)));
        }
        let unit = self.report.package.unit(self.state.file);
        for wrapper in unit.configured.ast.list(fields).iter().rev().copied() {
            if !self.active(wrapper) {
                continue;
            }
            let mut field_node = wrapper;
            while let Kind::Attributed { value, .. } = self.kind(field_node) {
                field_node = value;
            }
            let Kind::FieldInit { name, .. } = self.kind(field_node) else {
                return Err(self.error(field_node, "invalid field initializer"));
            };
            let name = self.member_name(field_node, name)?;
            let field = self
                .facts
                .members
                .field(variant_id, name)
                .ok_or_else(|| self.error(field_node, "unknown aggregate field"))?;
            self.accessible(field_node, field)?;
            if std::mem::replace(
                &mut self.state.field_seen[(field.0 - variant.start) as usize],
                true,
            ) {
                return Err(self.error(field_node, "duplicate field initializer"));
            }
            self.selected(field_node, Operation::FieldInit { field })?;
            let expected = self.instance_field(field_node, term, field)?;
            self.save(field_node, expected);
            self.state.work.push(Work::Enter(wrapper, Some(expected)));
        }
        if base.is_none() && self.state.field_seen.contains(&false) {
            return Err(self.error(node, "missing aggregate field initializer"));
        }
        Ok(())
    }
    pub(super) fn finish_struct(
        &mut self,
        node: NodeId,
        base: Option<NodeId>,
    ) -> Result<TermId, Error> {
        let term = self.term(node)?;
        if let Some(base) = base {
            self.coerce(base, term)?;
        }
        Ok(term)
    }
    pub(super) fn finish_field_init(
        &mut self,
        node: NodeId,
        value: NodeId,
    ) -> Result<TermId, Error> {
        let Some(Operation::FieldInit { .. }) =
            self.facts.node(self.key(node)).and_then(|f| f.operation)
        else {
            return Err(self.error(node, "missing selected initializer field"));
        };
        let expected = self.term(node)?;
        self.coerce(value, expected)?;
        Ok(expected)
    }
    pub(super) fn field(
        &mut self,
        node: NodeId,
        value: NodeId,
        name: Span,
    ) -> Result<TermId, Error> {
        let field = self.prepare_field(node, value, name)?;
        self.complete_field(node, field)
    }
    pub(super) fn prepare_field(
        &mut self,
        node: NodeId,
        value: NodeId,
        name: Span,
    ) -> Result<Field, Error> {
        let name = self.member_name(node, name)?;
        let receiver = self.autoderef(node, self.term(value)?)?;
        let kind = self.head(node, receiver)?.ok_or_else(|| {
            self.error(node, "field access requires a known tuple or struct type")
        })?;
        Ok(Field {
            receiver,
            name,
            kind,
        })
    }
    pub(super) fn field_dependency(&self, field: &Field) -> Option<crate::types::ItemId> {
        let (Constructor::Adt(item, _) | Constructor::AdtArgs(item, _)) = field.kind else {
            return None;
        };
        let definition = &self.report.resolution.definitions[item.0 as usize];
        (definition.kind == DefKind::Struct
            && definition
                .node
                .is_some_and(|key| self.facts.members.at_node(key).is_none()))
        .then_some(item)
    }
    pub(super) fn complete_field(&mut self, node: NodeId, field: Field) -> Result<TermId, Error> {
        let Field {
            receiver,
            name,
            kind,
        } = field;
        let (selection, ty) = match kind {
            Constructor::Tuple => {
                let Name::Index(index) = name else {
                    return Err(self.error(node, "tuple field requires a numeric index"));
                };
                let ty = self.child(node, receiver, index as usize)?;
                (Selection::Tuple(index), ty)
            }
            Constructor::Adt(item, _) | Constructor::AdtArgs(item, _) => {
                let variant = self.struct_variant(node, item)?;
                let field = self
                    .facts
                    .members
                    .field(variant, name)
                    .ok_or_else(|| self.error(node, "unknown aggregate field"))?;
                self.accessible(node, field)?;
                let ty = self.instance_field(node, receiver, field)?;
                (Selection::Nominal(field), ty)
            }
            _ => return Err(self.error(node, "field access requires a known tuple or struct type")),
        };
        self.selected(node, Operation::Field { field: selection })?;
        Ok(ty)
    }
    pub(super) fn array_index(
        &mut self,
        node: NodeId,
        value: NodeId,
        index: NodeId,
    ) -> Result<TermId, Error> {
        let array = self.autoderef(node, self.term(value)?)?;
        let operation = match self.head(node, array)? {
            Some(Constructor::Array(length)) => Some(Operation::Index { length }),
            Some(Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_)) => {
                self.state.pending_indexes.insert(node, array);
                None
            }
            Some(Constructor::Slice) => Some(Operation::IndexSlice),
            _ => return Err(self.error(node, "indexing requires a known array type")),
        };
        let usize = self.known(node, TypeId::primitive(Primitive::Usize))?;
        self.coerce(index, usize)?;
        if let Some(operation) = operation {
            self.selected(node, operation)?;
        }
        self.child(node, array, 0)
    }
    pub(super) fn publish_index_bounds(
        &mut self,
        solved: &crate::types::infer::Solved,
    ) -> Result<(), Error> {
        for (&node, &array) in &self.state.pending_indexes {
            let length =
                crate::types::length::Length::of(&self.facts.types, solved.types[array.0 as usize])
                    .map_err(|e| self.error(node, e))?;
            let key = self.key(node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| {
                    super::Declarations::error(self.report, key, "missing index selection")
                })?
                .operation = Some(match length {
                crate::types::length::Length::Known(length) => Operation::Index { length },
                crate::types::length::Length::Parameter(length) => {
                    Operation::IndexParameter { length }
                }
            });
        }
        self.state.pending_indexes.clear();
        Ok(())
    }
    pub(super) fn assignment_target(
        &self,
        node: NodeId,
        left: NodeId,
    ) -> Result<Option<DefId>, Error> {
        if self.state.pending_indexes.contains_key(&left)
            || self.facts.node(self.key(left)).is_some_and(|f| {
                matches!(
                    f.operation,
                    Some(
                        Operation::Field { .. }
                            | Operation::Index { .. }
                            | Operation::IndexParameter { .. }
                            | Operation::IndexSlice
                            | Operation::Dereference
                    )
                )
            })
        {
            return Ok(None);
        }
        let Some(Target::Definition(target)) =
            self.facts.node(self.key(left)).and_then(|f| f.reference)
        else {
            return Err(self.error(node, "assignment requires a place"));
        };
        if !matches!(
            self.report.resolution.definitions[target.0 as usize].kind,
            DefKind::Local | DefKind::Parameter
        ) {
            return Err(self.error(node, "assignment requires a local place"));
        }
        Ok(Some(target))
    }
}
