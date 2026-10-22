//! Range syntax uses its canonical record and the ordinary inferred element binding.
use super::{Checker, Domain, Error, NodeId, Operation, TermId, Type, Work};
use crate::{
    support::library_types::LibraryType,
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
