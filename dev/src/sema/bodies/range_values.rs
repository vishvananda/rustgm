//! Range syntax uses its canonical record and the ordinary inferred element binding.
use super::{Checker, Domain, Error, NodeId, Operation, TermId, Type, Work};
use crate::{
    sema::members::Name,
    support::{bindings::LangItem, library_types::LibraryType},
    types::{
        ItemId,
        infer::{Instantiation, arguments::Argument},
        substitute::Bound,
    },
};
impl Checker<'_> {
    pub(super) fn enter_range(
        &mut self,
        node: NodeId,
        inclusive: bool,
        start: Option<NodeId>,
        end: Option<NodeId>,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let modern = self
            .report
            .package
            .requested_new_range()
            .map_err(|e| self.error(node, e))?;
        if self.facts.source_bindings.is_some() {
            return self.enter_source_range(node, inclusive, start, end, modern, expected);
        }
        if !modern
            && !inclusive
            && let (Some(start), Some(end)) = (start, end)
        {
            return self.enter_bootstrap_range(node, start, end, expected);
        }
        Err(self.error(
            node,
            "this range form requires actual source range declarations",
        ))
    }
    fn enter_source_range(
        &mut self,
        node: NodeId,
        inclusive: bool,
        start: Option<NodeId>,
        end: Option<NodeId>,
        modern: bool,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let role = match (start.is_some(), end.is_some(), inclusive, modern) {
            (_, false, true, _) => return Err(self.error(node, "inclusive range requires an end")),
            (true, true, true, false) => {
                return Err(self.error(
                    node,
                    "source inclusive range constructor calls require their actual call plan",
                ));
            }
            (false, false, false, _) => LangItem::RangeFull,
            (true, false, false, false) => LangItem::RangeFrom,
            (true, false, false, true) => LangItem::RangeFromCopy,
            (false, true, false, _) => LangItem::RangeTo,
            (true, true, false, false) => LangItem::Range,
            (true, true, false, true) => LangItem::RangeCopy,
            (false, true, true, false) => LangItem::RangeToInclusive,
            (false, true, true, true) => LangItem::RangeToInclusiveCopy,
            (true, true, true, true) => LangItem::RangeInclusiveCopy,
        };
        let catalog = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error(node, "source range lacks its actual catalog"))?;
        catalog
            .require_provider_report(self.report)
            .map_err(|e| self.error(node, e))?;
        let declaration = catalog
            .validated_language(role)
            .map_err(|e| self.error(node, e))?
            .ok_or_else(|| {
                self.error(
                    node,
                    "range syntax requires its actual source language struct",
                )
            })?;
        crate::sema::header_attributes::charge_source_contract_query(self.facts, 6)?;
        let owner = ItemId(declaration.definition.0);
        let [variant] = self.facts.members.variants(owner) else {
            return Err(self.error(node, "source range requires its actual struct variant"));
        };
        let variant = *variant;
        let record = self.facts.members.variants[variant.0 as usize];
        if (record.end - record.start) as usize
            != usize::from(start.is_some()) + usize::from(end.is_some())
        {
            return Err(self.error(node, "range construction must initialize all actual fields"));
        }
        let term = self.implicit_nominal(node, owner, expected)?;
        if let Some(expected) = expected {
            self.shape_equation(node, term, expected)?;
        }
        self.save(node, term);
        self.selected(node, Operation::Construct { variant })?;
        let end_name = if modern && inclusive { "last" } else { "end" };
        // Work is pushed backwards; runtime evaluation remains start, then end.
        for (operand, name) in [(end, end_name), (start, "start")] {
            if let Some(operand) = operand {
                let name = self.facts.members.fields[record.start as usize..record.end as usize]
                    .iter()
                    .find_map(|field| match field.name {
                        Name::Named(id) if self.facts.members.names.get(id) == name => {
                            Some(field.name)
                        }
                        _ => None,
                    })
                    .ok_or_else(|| {
                        self.error(node, "range construction has no matching named field")
                    })?;
                let span = self
                    .report
                    .package
                    .unit(self.state.file)
                    .configured
                    .ast
                    .node(operand)
                    .span;
                let field = self.select_field(
                    node,
                    variant,
                    name,
                    span,
                    "range construction has no matching named field",
                )?;
                let expected = self.instance_field(node, term, field)?;
                self.state.work.push(Work::CoerceElement(operand, expected));
                self.state.work.push(Work::Enter(operand, Some(expected)));
            }
        }
        Ok(())
    }

    fn enter_bootstrap_range(
        &mut self,
        node: NodeId,
        start: NodeId,
        end: NodeId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let nominal = self
            .report
            .resolution
            .library_types
            .get(LibraryType::Range)
            .ok_or_else(|| self.error(node, "missing canonical Range identity"))?;
        let owner = ItemId(nominal.definition.0);
        let [parameter] = nominal.parameters.as_slice() else {
            return Err(self.error(node, "invalid Range parameter schema"));
        };
        let Some(Type::Parameter(parameter)) =
            self.facts.definitions[parameter.0 as usize].and_then(|ty| self.facts.types.kind(ty))
        else {
            return Err(self.error(node, "missing Range parameter"));
        };
        let [variant] = self.facts.members.variants(owner) else {
            return Err(self.error(node, "missing Range record"));
        };
        let variant = *variant;
        let element = self.variable(node, Domain::Any)?;
        let arguments = [Argument::Type(element)];
        let term = self
            .state
            .infer
            .nominal(owner, &arguments)
            .map_err(|e| self.error(node, e))?;
        if let Some(expected) = expected {
            self.shape_equation(node, term, expected)?;
        }
        let fields = crate::sema::annotations::Fields {
            regions: Bound::new(Vec::new()),
            parameters: Instantiation::new(
                &mut self.state.infer,
                &self.facts.types,
                &[(parameter, Argument::Type(element))],
            )
            .map_err(|e| self.error(node, e))?,
        };
        self.state.call_types.insert_nominal(term, fields);
        self.save(node, term);
        self.selected(node, Operation::Construct { variant })?;
        self.state.work.push(Work::CoerceElement(end, element));
        self.state.work.push(Work::Enter(end, Some(element)));
        self.state.work.push(Work::CoerceElement(start, element));
        self.state.work.push(Work::Enter(start, Some(element)));
        Ok(())
    }
}
