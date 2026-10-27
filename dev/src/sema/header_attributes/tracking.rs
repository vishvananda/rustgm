//! Caller ABI policy belongs to the actual completed source header.
use super::{Declarations, DefId, DefKind, Error, Header, NodeKey, Report, charge_query, ledger, validate_header};
use crate::syntax::{ast::Kind, attribute::{Attribute, Input}, item::{Abi, ItemKind}};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Tracking {
    Untracked,
    Explicit,
    Inherited(DefId),
}

pub(in crate::sema) fn consume_tracking(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
    tracking: bool,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error("caller and intrinsic const policy require word attributes"));
    }
    let unit = report.package.unit(declaration.file);
    let Kind::Item(index) = unit.configured.ast.node(declaration.node).kind else {
        return Err(error("caller policy requires an actual function declaration"));
    };
    let ItemKind::Function { abi: Abi::Rust, flags, .. } = unit.configured.ast.items[index as usize].kind else {
        return Err(error("track_caller requires the Rust ABI"));
    };
    if flags.asynchronous {
        return Err(error("async caller tracking is not implemented"));
    }
    if report.package.source_core().is_none() {
        return Err(error("ordinary caller tracking requires a checked source header"));
    }
    if !tracking {
        if !flags.constant || !report.package.requested_internal_attributes() {
            return Err(error("intrinsic const policy requires a const function and rustc_attrs"));
        }
        // The source intrinsic producer checks the actual attributed declaration;
        // this marker alone cannot turn an ordinary function into an intrinsic.
        return Ok(());
    }
    let header = ledger(facts, 0)?.local.declarations.get_mut(&declaration)
        .ok_or_else(|| error("caller policy lacks its actual retained source header"))?;
    header.tracking = Tracking::Explicit;
    Ok(())
}

impl Declarations {
    pub(crate) fn charge_caller_inputs(&self, work: usize, bytes: usize) -> Result<(), &'static str> {
        super::charge_source_retention(self, work, bytes).map_err(|_| "caller inputs exceed source work/storage budget")
    }

    /// Header data only; callers still prove the exact call/body/instance.
    pub(crate) fn recorded_caller_tracking(&self, definition: DefId) -> Result<bool, &'static str> {
        if matches!(self.declaration_header(definition)?.map(|header| header.declaration),
            Some(crate::resolve::body_origin::DeclarationOrigin::Imported(_))) {
            super::super::imported::caller_tracking_absent(self, definition)?;
            return Ok(false);
        }
        if self.source_bindings.is_none() { return Ok(false); }
        let Some(headers) = self.must_use_contracts.as_deref()
            .and_then(|owner| owner.headers.as_deref()).map(|ledger| &ledger.local)
        else {
            if self.source_bindings.is_none() { return Ok(false); }
            return Err("caller tracking lacks completed source header data");
        };
        charge_query(headers, 1).map_err(|_| "caller header work budget exceeded")?;
        if let Some(key) = headers.definitions.get(&definition) {
            if !headers.complete { return Err("caller tracking source header is incomplete"); }
            let header = headers.declarations.get(key).ok_or("caller header identity disappeared")?;
            return Ok(header.tracking != Tracking::Untracked);
        }
        if self.function_indices.contains_key(&definition) {
            return Err("source function lacks its caller policy header");
        }
        Err("imported caller tracking policy is unknown")
    }
    pub(crate) fn checked_caller_tracking(&self, report: &Report, definition: DefId) -> Result<bool, Error> {
        let actual = report.resolution.definitions.get(definition.0 as usize)
            .ok_or_else(|| Error::unlocated("caller policy has an invalid definition"))?;
        if actual.kind != DefKind::Function { return Err(Error::unlocated("caller policy requires a function")); }
        let Some(key) = actual.node else {
            if super::super::bootstrap_owner(report, definition)? { return Ok(false); }
            super::super::imported::caller_tracking_absent(self, definition).map_err(Error::unlocated)?;
            return Ok(false);
        };
        if report.package.source_core().is_none() { return Ok(false); }
        let headers = self.must_use_contracts.as_deref().and_then(|owner| owner.headers.as_deref())
            .map(|ledger| &ledger.local).filter(|headers| headers.complete)
            .ok_or_else(|| Error::unlocated("caller source headers are incomplete"))?;
        charge_query(headers, 1)?;
        let header = headers.definitions.get(&definition).filter(|node| **node == key)
            .and_then(|_| headers.declarations.get(&key))
            .ok_or_else(|| Error::unlocated("caller policy changed declaration identity"))?;
        validate_header(report, self, definition, key, header)?;
        Ok(header.tracking != Tracking::Untracked)
    }
}

/// Called only after the ordinary canonical override conformance succeeds.
pub(in crate::sema) fn inherit_tracking(
    report: &Report,
    facts: &mut Declarations,
    expected: DefId,
    actual: DefId,
) -> Result<(), Error> {
    if !facts.checked_caller_tracking(report, expected)? { return Ok(()); }
    let declaration = report.resolution.definitions.get(actual.0 as usize)
        .filter(|definition| definition.kind == DefKind::Function).and_then(|definition| definition.node)
        .ok_or_else(|| Error::unlocated("inherited caller policy needs its actual source method"))?;
    let header: &mut Header = ledger(facts, 0)?.local.declarations.get_mut(&declaration)
        .ok_or_else(|| Error::unlocated("inherited caller policy lacks its actual header"))?;
    if header.tracking == Tracking::Untracked { header.tracking = Tracking::Inherited(expected); }
    Ok(())
}
