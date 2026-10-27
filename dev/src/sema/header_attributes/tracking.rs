//! Caller ABI policy belongs to the actual completed source header.
use super::{
    Declarations, DefId, DefKind, Error, Header, NodeKey, Report, charge_query, ledger,
    validate_header,
};
use crate::support::decorations::Consumer;
use crate::syntax::{
    ast::Kind,
    attribute::{Attribute, Input},
    item::ItemKind,
};

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
    policy: Consumer,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error(
            "caller and intrinsic const policy require word attributes",
        ));
    }
    let unit = report.package.unit(declaration.file);
    let Kind::Item(index) = unit.configured.ast.node(declaration.node).kind else {
        return Err(error(
            "caller policy requires an actual function declaration",
        ));
    };
    let ItemKind::Function { abi, flags, .. } = unit.configured.ast.items[index as usize].kind
    else {
        return Err(error("caller policy requires a function"));
    };
    if policy == Consumer::CallerTracking
        && crate::sema::declarations::callable_convention(report, declaration, abi)?.abi
            != crate::types::callable::Abi::Rust
    {
        return Err(error("track_caller requires the Rust ABI"));
    }
    if flags.asynchronous {
        return Err(error("async caller tracking is not implemented"));
    }
    if report.package.source_core().is_none() {
        return Err(error(
            "ordinary caller tracking requires a checked source header",
        ));
    }
    if policy != Consumer::CallerTracking {
        if !report.package.requested_internal_attributes() {
            return Err(error("intrinsic policy requires rustc_attrs"));
        }
        // The source intrinsic producer checks the actual attributed declaration;
        // this marker alone cannot turn an ordinary function into an intrinsic.
        return Ok(());
    }
    let header = ledger(facts, 0)?
        .local
        .declarations
        .get_mut(&declaration)
        .ok_or_else(|| error("caller policy lacks its actual retained source header"))?;
    header.tracking = Tracking::Explicit;
    Ok(())
}

impl Declarations {
    pub(crate) fn charge_caller_inputs(
        &self,
        work: usize,
        bytes: usize,
    ) -> Result<(), &'static str> {
        super::charge_source_retention(self, work, bytes)
            .map_err(|_| "caller inputs exceed source work/storage budget")
    }

    /// Header data only; callers still prove the exact call/body/instance.
    pub(crate) fn recorded_caller_tracking(&self, definition: DefId) -> Result<bool, &'static str> {
        if self.constructor_tracking_absent(definition)? {
            return Ok(false);
        }
        let origin = self
            .declaration_header(definition)?
            .map(|header| header.declaration);
        if matches!(
            origin,
            Some(crate::resolve::body_origin::DeclarationOrigin::Imported(_))
        ) {
            super::super::imported::caller_tracking_absent(self, definition)?;
            return Ok(false);
        }
        if self.source_bindings.is_none() {
            return Ok(false);
        }
        if matches!(
            origin,
            Some(crate::resolve::body_origin::DeclarationOrigin::Source(_))
        ) && self
            .source_bindings
            .as_deref()
            .is_some_and(crate::sema::compiler_items::Catalog::local_caller_tracking_closed)
        {
            return Ok(false);
        }
        let Some(headers) = self
            .must_use_contracts
            .as_deref()
            .and_then(|owner| owner.headers.as_deref())
            .map(|ledger| &ledger.local)
        else {
            if self.source_bindings.is_none() {
                return Ok(false);
            }
            return Err("caller tracking lacks completed source header data");
        };
        charge_query(headers, 1).map_err(|_| "caller header work budget exceeded")?;
        if let Some(key) = headers.definitions.get(&definition) {
            if !headers.complete {
                return Err("caller tracking source header is incomplete");
            }
            let header = headers
                .declarations
                .get(key)
                .ok_or("caller header identity disappeared")?;
            return Ok(header.tracking != Tracking::Untracked);
        }
        if self.function_indices.contains_key(&definition) {
            return Err("source function lacks its caller policy header");
        }
        Err("imported caller tracking policy is unknown")
    }
    pub(crate) fn checked_caller_tracking(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<bool, Error> {
        let actual = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| Error::unlocated("caller policy has an invalid definition"))?;
        if actual.kind != DefKind::Function {
            if matches!(actual.kind, DefKind::Constructor | DefKind::Variant)
                && actual.namespace == crate::resolve::model::Namespace::Value
                && self
                    .constructor_tracking_absent(definition)
                    .map_err(Error::unlocated)?
            {
                return Ok(false);
            }
            return Err(Error::unlocated("caller policy requires a function"));
        }
        let Some(key) = actual.node else {
            if super::super::bootstrap_owner(report, definition)? {
                return Ok(false);
            }
            super::super::imported::caller_tracking_absent(self, definition)
                .map_err(Error::unlocated)?;
            return Ok(false);
        };
        if report.package.source_core().is_none() {
            return Ok(false);
        }
        let headers = self
            .must_use_contracts
            .as_deref()
            .and_then(|owner| owner.headers.as_deref())
            .map(|ledger| &ledger.local)
            .filter(|headers| headers.complete)
            .ok_or_else(|| Error::unlocated("caller source headers are incomplete"))?;
        charge_query(headers, 1)?;
        let header = headers
            .definitions
            .get(&definition)
            .filter(|node| **node == key)
            .and_then(|_| headers.declarations.get(&key))
            .ok_or_else(|| Error::unlocated("caller policy changed declaration identity"))?;
        validate_header(report, self, definition, key, header)?;
        Ok(header.tracking != Tracking::Untracked)
    }

    /// A canonical nominal constructor is callable without a function header.
    fn constructor_tracking_absent(&self, definition: DefId) -> Result<bool, &'static str> {
        let Some(variant) = self.members.constructor(definition) else {
            return Ok(false);
        };
        let variant = self
            .members
            .variants
            .get(variant.0 as usize)
            .ok_or("caller constructor has no actual variant")?;
        let Some(crate::types::Kind::FunctionItem {
            item, signature, ..
        }) = self
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .and_then(|ty| self.types.kind(ty))
        else {
            return Err("caller constructor has no canonical callable");
        };
        if item.0 != definition.0
            || self.function_indices.contains_key(&definition)
            || !matches!(self.types.kind(signature), Some(crate::types::Kind::Function { result, .. })
                if matches!(self.types.kind(result), Some(crate::types::Kind::Adt(owner, ..)) if owner == variant.owner))
        {
            return Err("caller constructor changed its actual callable or nominal owner");
        }
        Ok(true)
    }
}

/// Called only after the ordinary canonical override conformance succeeds.
pub(in crate::sema) fn inherit_tracking(
    report: &Report,
    facts: &mut Declarations,
    expected: DefId,
    actual: DefId,
) -> Result<(), Error> {
    if !facts.checked_caller_tracking(report, expected)? {
        return Ok(());
    }
    let declaration = report
        .resolution
        .definitions
        .get(actual.0 as usize)
        .filter(|definition| definition.kind == DefKind::Function)
        .and_then(|definition| definition.node)
        .ok_or_else(|| {
            Error::unlocated("inherited caller policy needs its actual source method")
        })?;
    let header: &mut Header = ledger(facts, 0)?
        .local
        .declarations
        .get_mut(&declaration)
        .ok_or_else(|| Error::unlocated("inherited caller policy lacks its actual header"))?;
    if header.tracking == Tracking::Untracked {
        header.tracking = Tracking::Inherited(expected);
    }
    Ok(())
}

pub(super) fn validate_intrinsic_annotations(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    declaration: NodeKey,
) -> Result<(), Error> {
    let Some(header) = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.local.declarations.get(&declaration))
    else {
        return Ok(());
    };
    let unit = report.package.unit(declaration.file);
    for owner in &header.owners {
        super::charge_source_contract_query(facts, owner.effects.len())?;
        for (attribute, forest) in unit.retained_attributes(owner.effects.clone()) {
            match attribute.name(forest, unit.source.text()) {
                Some("rustc_intrinsic_const_stable_indirect")
                    if report.resolution.scopes
                        [report.resolution.definitions[definition.0 as usize].scope.0 as usize]
                        .associated =>
                {
                    return Err(Error::at(
                        declaration.file,
                        attribute.span.start,
                        "intrinsic const policy requires a free function",
                    ));
                }
                Some("rustc_nounwind")
                    if report
                        .resolution
                        .source_bindings
                        .as_deref()
                        .and_then(|reserved| reserved.intrinsic(definition))
                        .is_none_or(|reserved| reserved.declaration.origin != declaration) =>
                {
                    return Err(Error::at(
                        declaration.file,
                        attribute.span.start,
                        "ordinary source nounwind callable execution is not implemented",
                    ));
                }
                _ => {}
            }
        }
    }
    Ok(())
}
impl Declarations {
    pub(crate) fn intrinsic_const_stable_indirect(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<bool, Error> {
        let _ = self.checked_caller_tracking(report, definition)?;
        let actual = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .and_then(|definition| definition.node)
            .ok_or_else(|| {
                Error::unlocated("intrinsic const policy has no actual source declaration")
            })?;
        let header = self
            .must_use_contracts
            .as_deref()
            .and_then(|owner| owner.headers.as_deref())
            .and_then(|ledger| ledger.local.declarations.get(&actual))
            .ok_or_else(|| Error::unlocated("intrinsic const policy lost its completed header"))?;
        let unit = report.package.unit(actual.file);
        for owner in &header.owners {
            super::charge_source_contract_query(self, owner.effects.len())?;
            for (attribute, forest) in unit.retained_attributes(owner.effects.clone()) {
                if attribute.name(forest, unit.source.text())
                    == Some("rustc_intrinsic_const_stable_indirect")
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/tracked_calls.rs"]
mod tests;
