//! Index checked call syntax by the actual installed external function.
use super::{ColdImports, DefId, Error, HashMap, Imported, ReservationWork, error};
use crate::{
    resolve::unit::NodeKey,
    support::{crates::generics::source::ValidatedView, retained},
    syntax::ast::List,
};

#[derive(Default)]
pub(super) struct Calls {
    headers: HashMap<DefId, Header>,
    arguments: HashMap<NodeKey, List>,
}

pub(in crate::resolve) struct Header {
    pub positions: Box<[u32]>,
    pub runtime: u32,
}

impl Imported {
    pub(super) fn prepare_legacy_const_calls(
        &mut self,
        rows: ValidatedView<'_>,
        definitions: &[DefId],
        work: &mut ReservationWork,
    ) -> Result<Vec<(DefId, Header)>, Error> {
        let Some(attributes) = rows.header_attributes() else {
            return Ok(Vec::new());
        };
        work.charge(attributes.functions.len())?;
        let mut marked = Vec::new();
        for (contract, function) in attributes.functions.iter().zip(&rows.functions) {
            let positions = contract.legacy_const_positions();
            if positions.is_empty() {
                continue;
            }
            let definition = definitions[function.declaration.0 as usize];
            let parameters = function.callable.parameters;
            work.reserve(size_of::<(DefId, Header)>() * 4 + size_of_val(positions))?;
            marked.push((
                definition,
                Header {
                    positions: positions.into(),
                    runtime: parameters.end - parameters.start,
                },
            ));
        }
        if !marked.is_empty() {
            let cold = self.cold.get_or_insert_with(Box::default);
            if cold.legacy.is_none() {
                work.reserve(size_of::<Calls>())?;
            }
            let calls = cold.legacy.get_or_insert_with(Box::default);
            let growth = retained::Growth::map::<DefId, Header>(
                calls.headers.len(),
                calls.headers.capacity(),
                marked.len(),
            );
            work.charge(growth.work)?;
            work.reserve(growth.bytes)?;
            calls
                .headers
                .try_reserve(marked.len())
                .map_err(|_| error("cannot reserve imported legacy const functions"))?;
        }
        Ok(marked)
    }

    pub(super) fn install_legacy_const_calls(&mut self, marked: Vec<(DefId, Header)>) {
        if let Some(calls) = self
            .cold
            .as_deref_mut()
            .and_then(|cold| cold.legacy.as_deref_mut())
        {
            calls.headers.extend(marked);
        }
    }

    pub(in crate::resolve) fn has_legacy_const_calls(&self) -> bool {
        self.legacy_const_calls()
            .is_some_and(|calls| !calls.headers.is_empty())
    }

    pub(in crate::resolve) fn legacy_const_header(&self, definition: DefId) -> Option<&Header> {
        self.legacy_const_calls()?.headers.get(&definition)
    }

    pub(in crate::resolve) fn record_legacy_const_arguments(
        &mut self,
        callee: NodeKey,
        arguments: List,
        written: u32,
    ) -> Result<(), Error> {
        let Some(ColdImports {
            work,
            legacy: Some(calls),
            ..
        }) = self.cold.as_deref_mut()
        else {
            return Err(error("missing imported legacy const state"));
        };
        let growth = retained::Growth::map::<NodeKey, List>(
            calls.arguments.len(),
            calls.arguments.capacity(),
            1,
        );
        work.charge(growth.work.saturating_add(written as usize))?;
        work.reserve(growth.bytes.saturating_add(
            (written as usize).saturating_mul(size_of::<crate::syntax::ast::NodeId>()),
        ))?;
        calls
            .arguments
            .try_reserve(1)
            .map_err(|_| error("cannot reserve a legacy const call"))?;
        calls.arguments.insert(callee, arguments);
        Ok(())
    }

    pub(crate) fn legacy_const_arguments(&self, callee: NodeKey) -> Option<List> {
        self.legacy_const_calls()?.arguments.get(&callee).copied()
    }

    /// Explain a failed constant operand at its original source site.
    pub(crate) fn is_legacy_const_operand(
        &self,
        package: &crate::resolve::package::Package,
        operand: NodeKey,
    ) -> bool {
        let ast = &package.unit(operand.file).configured.ast;
        self.legacy_const_calls().is_some_and(|calls| {
            calls.arguments.iter().any(|(callee, arguments)| {
                callee.file == operand.file && ast.list(*arguments).contains(&operand.node)
            })
        })
    }

    fn legacy_const_calls(&self) -> Option<&Calls> {
        self.cold.as_deref()?.legacy.as_deref()
    }
}
