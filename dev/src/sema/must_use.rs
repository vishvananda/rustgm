//! Source-owned diagnostic contracts do not grant execution or trait permission.
mod declarations;
mod messages;
pub(super) use declarations::collect;
pub(in crate::sema) mod discarded_types;
mod removed_owners;
#[cfg(test)]
#[path = "../../tests/invariants/safety_diagnostics.rs"]
mod safety_tests;
mod supertraits;
mod trivial_types;
use super::{model::Declarations, safety::Level};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey},
    syntax::token_tree::Span,
};

pub use crate::{resolve::package::Warning, support::diagnostics::WarningCode};
struct Contract {
    image: std::sync::Arc<crate::support::source::OriginalImage>,
    owner: NodeKey,
    declaration: NodeKey,
    span: Span,
    effect: usize,
    reason: Option<Box<str>>,
}
/// Borrowed local facts; an imported unknown is an error, never an empty view.
pub struct ContractView<'a> {
    pub origin: super::header_attributes::ContractOrigin<'a>,
    pub reason: Option<&'a str>,
}
#[derive(Clone, Copy, Default)]
pub struct Work {
    pub attributes: usize,
    pub declarations: usize,
    pub statements: usize,
    pub type_steps: usize,
    pub trait_steps: usize,
    pub retained_contracts: usize,
    pub retained_reasons: usize,
    pub warnings: usize,
    pub inhabitation_steps: usize,
    pub inhabitation_queries: usize,
    pub contract_input_steps: usize,
    pub contract_input_bytes: usize,
    pub message_steps: usize,
    pub message_bytes: usize,
}
#[derive(Default)]
pub(super) struct Contracts {
    pub(super) cfi_encodings: Option<Box<HashMap<DefId, Box<str>>>>,
    pub(super) ffi_effects: Option<Box<HashMap<DefId, super::foreign::MemoryEffect>>>,
    pub(super) used_statics: Option<Box<super::used_statics::Retention>>,
    pub(in crate::sema) headers: Option<Box<super::header_attributes::Ledger>>,
    pub(in crate::sema) specialization: Option<Box<super::specialization::LocalMarkers>>,
    pub(in crate::sema) non_exhaustive: Option<Box<super::non_exhaustive::Contracts>>,
    pub(in crate::sema) inline: Option<Box<super::inlining::Contracts>>,
    pub(in crate::sema) parentheses: Option<Box<super::parentheses::Contracts>>,
    pub(in crate::sema) diagnostics: Option<Box<super::diagnostic_contracts::Contracts>>,
    trait_requirements:
        std::cell::RefCell<HashMap<crate::types::ItemId, Option<discarded_types::Requirement>>>,
    trait_steps: std::cell::Cell<usize>,
    pub(super) trait_bytes: std::cell::Cell<usize>,
    type_requirements: HashMap<crate::types::TypeId, discarded_types::TypeResult>,
    trivial: HashMap<
        (crate::resolve::package::ModuleId, crate::types::TypeId),
        Result<bool, &'static str>,
    >,
    trivial_bytes: usize,
    sources: Vec<Contract>,
    definitions: HashMap<DefId, usize>,
    pub(in crate::sema) unknown: HashSet<DefId>,
    warnings: Vec<Warning>,
    warning_bytes: usize,
    warning_steps: usize,
    diagnosed: HashSet<(WarningCode, NodeKey, u32, u32)>,
    pub(in crate::sema) work: Work,
}
impl Declarations {
    pub(in crate::sema) fn retain_source_warning(
        &mut self,
        key: NodeKey,
        warning: Warning,
    ) -> Result<(), Error> {
        let offset = warning.diagnostic.span.start;
        self.must_use_contracts
            .get_or_insert_default()
            .record_warning(key, offset, 0, warning.code, warning.diagnostic)
    }
    /// Read retained warnings without draining them or repeating semantic work.
    pub fn warnings(&self) -> impl Iterator<Item = &Warning> {
        self.must_use_contracts
            .iter()
            .flat_map(|contracts| contracts.warnings.iter())
    }
    /// Move the existing sink at a terminal failure; borrowed success views stay persistent.
    #[must_use]
    pub fn retain_error_warnings(&mut self, error: Error) -> Error {
        let Some(contracts) = self.must_use_contracts.as_deref_mut() else {
            return error;
        };
        error.retain_warnings(std::mem::take(&mut contracts.warnings))
    }
    #[must_use]
    pub fn must_use_work(&self) -> Work {
        self.must_use_contracts
            .as_deref()
            .map_or_else(Work::default, |contracts| Work {
                trait_steps: contracts.trait_steps.get(),
                ..contracts.work
            })
    }
    /// Query the actual declaration; source-free imported headers require transport.
    /// # Errors
    /// Rejects unknown identities, changed local owners and untransported contracts.
    pub fn must_use_contract(
        &self,
        report: &Report,
        definition: DefId,
    ) -> Result<Option<ContractView<'_>>, Error> {
        let actual = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "unknown must_use declaration",
                )
            })?;
        if actual.node.is_none() {
            use super::header_attributes::ContractKnowledge;
            if super::header_attributes::local_crate_owner(report, definition)?
                || super::header_attributes::bootstrap_owner(report, definition)?
            {
                return Ok(None);
            }
            if super::header_attributes::has_imported(self, definition) {
                return match self
                    .header_attribute_contracts(report, definition)?
                    .must_use
                {
                    ContractKnowledge::KnownAbsent => Ok(None),
                    ContractKnowledge::KnownPresent(view) => Ok(Some(view)),
                    ContractKnowledge::Unknown => Err(Error::at(
                        crate::resolve::unit::FileId(0),
                        0,
                        "imported must_use contract transport is not implemented",
                    )),
                };
            }
            return Err(Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "imported must_use contract transport is not implemented",
            ));
        }
        let Some(contracts) = self.must_use_contracts.as_deref() else {
            return Ok(None);
        };
        if contracts.unknown.contains(&definition) {
            return Err(Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "imported must_use contract transport is not implemented",
            ));
        }
        let Some(&index) = contracts.definitions.get(&definition) else {
            return Ok(None);
        };
        let contract = &contracts.sources[index];
        let unit = report.package.unit(contract.owner.file);
        let owner = unit
            .owner_indices
            .get(contract.owner.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1));
        if !std::sync::Arc::ptr_eq(&contract.image, unit.source.original_image())
            || actual.node != Some(contract.declaration)
            || owner.is_none_or(|owner| {
                !owner.enabled
                    || owner.node != contract.owner.node
                    || !owner.attributes.contains(&contract.effect)
                    || unit
                        .retained_attributes(contract.effect..contract.effect + 1)
                        .next()
                        .is_none_or(|(attribute, forest)| {
                            attribute.span != contract.span
                                || attribute.name(forest, unit.source.text()) != Some("must_use")
                        })
            })
        {
            return Err(Error::at(
                contract.owner.file,
                contract.span.start,
                "must_use contract actual declaration owner changed",
            ));
        }
        Ok(Some(ContractView {
            origin: super::header_attributes::ContractOrigin::source_site(
                contract.owner,
                contract.declaration,
                contract.span,
            ),
            reason: contract.reason.as_deref(),
        }))
    }
    pub(super) fn must_use_diagnostic(
        &mut self,
        report: &Report,
        key: NodeKey,
        offset: u32,
        issue: u32,
        code: WarningCode,
        message: String,
    ) -> Result<(), Error> {
        self.must_use_diagnostic_in(
            report,
            key,
            offset,
            issue,
            code,
            (message, crate::resolve::model::ExpansionId(0)),
        )
    }
    pub(in crate::sema) fn must_use_diagnostic_in(
        &mut self,
        report: &Report,
        key: NodeKey,
        offset: u32,
        issue: u32,
        code: WarningCode,
        message: (String, crate::resolve::model::ExpansionId),
    ) -> Result<(), Error> {
        self.must_use_diagnostic_at(report, (key, key), offset, issue, code, message)
    }
    pub(in crate::sema) fn must_use_diagnostic_at(
        &mut self,
        report: &Report,
        (scope, key): (NodeKey, NodeKey),
        offset: u32,
        issue: u32,
        code: WarningCode,
        message: (String, crate::resolve::model::ExpansionId),
    ) -> Result<(), Error> {
        if report.package.lint_cap().suppresses_lints() {
            return Ok(());
        }
        let level = self.safety_policies.warning_level(scope, code);
        if level == Level::Expect {
            let lint = crate::resolve::lint_policy::Lint::warning(code).ok_or_else(|| {
                Error::at(key.file, offset, "warning has no lint expectation consumer")
            })?;
            let id = self
                .safety_policies
                .expectation(scope, lint)
                .ok_or_else(|| {
                    Error::at(
                        key.file,
                        offset,
                        "effective lint expectation identity is missing",
                    )
                })?;
            report
                .lint_producer()
                .ok_or_else(|| Error::at(key.file, offset, "lint expectation producer is missing"))?
                .fulfill(
                    &report.package,
                    id,
                    key,
                    lint,
                    self.safety_policies
                        .progress
                        .as_deref_mut()
                        .ok_or_else(|| {
                            Error::at(key.file, offset, "lint request progress is missing")
                        })?,
                )?;
            return Ok(());
        }
        if level == Level::Allow {
            return Ok(());
        }
        let mut raw = Error::at(key.file, offset, message.0);
        raw.introduction = message.1;
        let diagnostic = report.diagnostic(raw);
        if level.denies() {
            return Err(diagnostic);
        }
        self.must_use_contracts
            .get_or_insert_default()
            .record_warning(key, offset, issue, code, diagnostic)
    }
}
impl super::safety::Policies {
    fn warning_level(&self, scope: NodeKey, code: WarningCode) -> Level {
        match code {
            WarningCode::Deprecated => self.deprecated(scope),
            WarningCode::DeprecatedInFuture => self.deprecated_in_future(scope),
            WarningCode::HiddenGlobReexports => self.hidden_glob_reexports(scope),
            WarningCode::IneffectiveUnstableTraitImpl => {
                self.ineffective_unstable_trait_impl(scope)
            }
            WarningCode::UnfulfilledLintExpectations => self.unfulfilled(scope),
            WarningCode::InvalidAtomicOrdering => self.atomic_ordering(scope),
            WarningCode::UnreachablePatterns => self.unreachable_patterns(scope),
            WarningCode::UnusedMustUse => self.must_use(scope),
            WarningCode::UnusedAttributes => self.unused_attributes(scope),
            WarningCode::InvalidDocAttributes => self.invalid_doc_attributes(scope),
            WarningCode::OverflowingLiterals => self.overflowing_literals(scope),
            WarningCode::ImplicitProvenanceCasts => self.implicit_provenance_casts(scope),
            WarningCode::UnreachablePub => self.unreachable_pub(scope),
            WarningCode::PtrToIntegerTransmuteInConsts => {
                self.ptr_to_integer_transmute_in_consts(scope)
            }
            WarningCode::MissingDebugImplementations => self.missing_debug_implementations(scope),
            WarningCode::MultipleSupertraitUpcastable => self.multiple_supertrait_upcastable(scope),
            WarningCode::MissingDocs => self.missing_docs(scope),
            WarningCode::Rust2021IncompatibleOrPatterns => {
                self.rust_2021_incompatible_or_patterns(scope)
            }
            WarningCode::UnusedVisibilities => self.unused_visibilities(scope),
            WarningCode::UnstableSyntaxPreExpansion => Level::Warn,
            WarningCode::DuplicateMacroAttributes => self.duplicate_macro_attributes(scope),
            WarningCode::AmbiguousDeriveHelpers => self.ambiguous_derive_helpers(scope),
            WarningCode::MalformedDiagnosticAttributes => self.diagnostic_attributes(scope, 0),
            WarningCode::MalformedDiagnosticFormatLiterals => self.diagnostic_attributes(scope, 1),
            WarningCode::MisplacedDiagnosticAttributes => self.diagnostic_attributes(scope, 2),
            WarningCode::UnsafeOpInUnsafeFn => self.operation(scope),
            WarningCode::StaticMutRefs => self.reference(scope),
            WarningCode::DangerousImplicitAutorefs => self.dangerous_autorefs(scope),
            WarningCode::DeprecatedSafe2024 => self.deprecated_safe_2024(scope),
        }
    }
}
impl Contracts {
    fn record_warning(
        &mut self,
        key: NodeKey,
        offset: u32,
        issue: u32,
        code: WarningCode,
        diagnostic: Error,
    ) -> Result<(), Error> {
        self.warning_steps = self.warning_steps.saturating_add(1);
        if self.warning_steps > 10_000_000 {
            return Err(Error::at(
                key.file,
                offset,
                "semantic warning work budget exceeded",
            ));
        }
        if !self.diagnosed.contains(&(code, key, offset, issue)) {
            let (steps, payload) = diagnostic.warning_payload().ok_or_else(|| {
                Error::at(
                    key.file,
                    offset,
                    "semantic warning has an invalid nested carrier",
                )
            })?;
            let slots =
                8 * size_of::<Warning>() + 4 * size_of::<(WarningCode, NodeKey, u32, u32)>();
            self.warning_bytes = self
                .warning_bytes
                .saturating_add(payload)
                .saturating_add(slots);
            self.warning_steps = self.warning_steps.saturating_add(steps);
            if self.warnings.len() >= 1_000_000
                || self.warning_bytes > 64 * 1024 * 1024
                || self.warning_steps > 10_000_000
            {
                return Err(Error::at(
                    key.file,
                    offset,
                    "semantic warning work or storage budget exceeded",
                ));
            }
            self.diagnosed.insert((code, key, offset, issue));
            self.warnings.push(Warning { code, diagnostic });
            self.work.warnings += 1;
        }
        Ok(())
    }
}
