//! Source-owned diagnostic contracts do not grant execution or trait permission.
mod declarations;
pub(super) use declarations::collect;
pub(in crate::sema) mod discarded_types;
mod supertraits;
mod removed_owners;
use super::{model::Declarations, safety::Level};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey},
    syntax::token_tree::Span,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WarningCode {
    UnusedMustUse,
    UnusedAttributes,
    DuplicateMacroAttributes,
}
impl WarningCode {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnusedMustUse => "unused_must_use",
            Self::UnusedAttributes => "unused_attributes",
            Self::DuplicateMacroAttributes => "duplicate_macro_attributes",
        }
    }
}
pub struct Warning {
    pub code: WarningCode,
    pub diagnostic: Error,
}
struct Contract {
    owner: NodeKey,
    declaration: NodeKey,
    span: Span,
    effect: usize,
    reason: Option<Box<str>>,
}
/// Borrowed local facts; an imported unknown is an error, never an empty view.
pub struct ContractView<'a> {
    pub owner: NodeKey,
    pub declaration: NodeKey,
    pub span: Span,
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
}
#[derive(Default)]
pub(super) struct Contracts {
    pub(in crate::sema) inline: Option<Box<super::inlining::Contracts>>,
    pub(in crate::sema) parentheses: Option<Box<super::parentheses::Contracts>>,
    trait_requirements:
        std::cell::RefCell<HashMap<crate::types::ItemId, Option<discarded_types::Requirement>>>,
    trait_steps: std::cell::Cell<usize>,
    type_requirements: HashMap<crate::types::TypeId, discarded_types::TypeResult>,
    sources: Vec<Contract>,
    definitions: HashMap<DefId, usize>,
    pub(in crate::sema) unknown: HashSet<DefId>,
    warnings: Vec<Warning>,
    diagnosed: HashSet<(WarningCode, NodeKey, u32, u32)>,
    pub(in crate::sema) work: Work,
}
impl Declarations {
    /// Read retained warnings without draining them or repeating semantic work.
    pub fn warnings(&self) -> impl Iterator<Item = &Warning> {
        self.must_use_contracts
            .iter()
            .flat_map(|contracts| contracts.warnings.iter())
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
        if actual.node != Some(contract.declaration)
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
            owner: contract.owner,
            declaration: contract.declaration,
            span: contract.span,
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
        let level = match code {
            WarningCode::UnusedMustUse => self.safety_policies.must_use(key),
            WarningCode::UnusedAttributes => self.safety_policies.unused_attributes(key),
            WarningCode::DuplicateMacroAttributes => self.safety_policies.duplicate_macro_attributes(key),
        };
        if level == Level::Allow {
            return Ok(());
        }
        let diagnostic = report.diagnostic(Error::at(key.file, offset, message));
        if level.denies() {
            return Err(diagnostic);
        }
        let contracts = self.must_use_contracts.get_or_insert_default();
        if contracts.diagnosed.insert((code, key, offset, issue)) {
            if contracts.warnings.len() >= 1_000_000 {
                return Err(Error::at(
                    key.file,
                    offset,
                    "must_use warning budget exceeded",
                ));
            }
            contracts.warnings.push(Warning { code, diagnostic });
            contracts.work.warnings += 1;
        }
        Ok(())
    }
}
