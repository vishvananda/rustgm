//! Shared immutable header data is distinct from each borrowed authentication.
use super::{Allowance, Declaration, Package, State, Unit, Witness, WitnessNode};
use crate::{
    expand::builtin::{Builtin, BuiltinKind},
    resolve::{bindings::FeatureName, package::Error},
    support::hash::HashSet,
    support::source::OriginalImage,
    syntax::attribute::Attribute,
    syntax::token_tree::Span,
};
use std::{rc::Rc, sync::Arc};

pub(in crate::resolve) struct HelperAttribute {
    name: Rc<str>,
    span: Span,
}
impl HelperAttribute {
    pub(super) fn new(name: Rc<str>, span: Span) -> Self {
        Self { name, span }
    }
    pub(in crate::resolve) fn name(&self) -> &str {
        &self.name
    }
    pub(in crate::resolve) const fn span(&self) -> Span {
        self.span
    }
}

pub(in crate::resolve) struct HeaderData {
    pub(super) declaration: Declaration,
    pub(super) witnesses: Vec<Witness>,
    pub(super) features: HashSet<Rc<str>>,
    pub(super) witness_work: usize,
    pub(super) lexeme_bytes: usize,
    pub(super) helpers: Vec<HelperAttribute>,
    pub(super) image: Arc<OriginalImage>,
}
impl HeaderData {
    pub(super) fn image_matches(&self, unit: &Unit) -> bool {
        Arc::ptr_eq(&self.image, unit.source.original_image())
    }
    #[cfg(test)]
    pub(in crate::resolve) fn feature_count(&self) -> usize {
        self.features.len()
    }
    pub(in crate::resolve) const fn declaration(&self) -> Declaration {
        self.declaration
    }
    pub(in crate::resolve) fn contains(&self, feature: &FeatureName<'_>) -> bool {
        self.features.contains(feature.spelling())
    }
    /// Read only after the existing selected-header authentication. This list
    /// does not itself authorize a helper on any attributed owner.
    pub(in crate::resolve) fn derive_helpers(
        &self,
        expected: Builtin,
    ) -> Result<&[HelperAttribute], Error> {
        if expected.kind() != BuiltinKind::Derive || self.declaration.builtin != Some(expected) {
            return Err(Error::at(
                self.declaration.node.file,
                0,
                "helper query requires its actual selected derive role",
            ));
        }
        Ok(&self.helpers)
    }
    pub(super) fn payload(&self) -> usize {
        size_of::<Self>()
            .saturating_add(
                self.witnesses
                    .capacity()
                    .saturating_mul(size_of::<Witness>()),
            )
            .saturating_add(
                self.witnesses
                    .iter()
                    .map(|witness| {
                        witness
                            .nodes
                            .capacity()
                            .saturating_mul(size_of::<WitnessNode>())
                    })
                    .sum::<usize>(),
            )
            .saturating_add(
                self.features
                    .capacity()
                    .saturating_mul(size_of::<Rc<str>>()),
            )
            .saturating_add(self.lexeme_bytes)
            .saturating_add(
                self.helpers
                    .capacity()
                    .saturating_mul(size_of::<HelperAttribute>()),
            )
    }
}

pub(super) enum Entry {
    Plain(Declaration),
    Owned(Rc<HeaderData>),
}
impl Entry {
    pub(super) fn declaration(&self) -> Declaration {
        match self {
            Self::Plain(declaration) => *declaration,
            Self::Owned(data) => data.declaration,
        }
    }
    pub(super) fn work(&self) -> usize {
        match self {
            Self::Plain(_) => 0,
            Self::Owned(data) => data.witness_work,
        }
    }
    pub(super) fn valid(&self, unit: &Unit) -> bool {
        match self {
            Self::Plain(_) => true,
            Self::Owned(data) => {
                data.image_matches(unit) && data.witnesses.iter().all(|witness| witness.valid(unit))
            }
        }
    }
    pub(super) fn shared(&self) -> Option<Rc<HeaderData>> {
        match self {
            Self::Plain(_) => None,
            Self::Owned(data) => Some(Rc::clone(data)),
        }
    }
    pub(super) fn payload(&self) -> usize {
        match self {
            Self::Plain(_) => 0,
            Self::Owned(data) => data.payload(),
        }
    }
}

pub(in crate::resolve) struct CheckedHeader<'package> {
    package: &'package Package,
    declaration: Declaration,
    data: Option<Rc<HeaderData>>,
}
impl<'package> CheckedHeader<'package> {
    pub(super) fn new(package: &'package Package, entry: &Entry) -> Self {
        Self {
            package,
            declaration: entry.declaration(),
            data: entry.shared(),
        }
    }
    pub(in crate::resolve) const fn summary(&self) -> Declaration {
        self.declaration
    }
    pub(in crate::resolve) fn into_data(self) -> Option<Rc<HeaderData>> {
        self.data
    }
    /// Declaration data only: a selected derive and its invocation owner must
    /// still be authenticated before a helper can be used.
    pub(in crate::resolve) fn derive_helpers(
        &self,
        expected: Builtin,
    ) -> Result<&[HelperAttribute], Error> {
        if expected.kind() != BuiltinKind::Derive || self.declaration.builtin != Some(expected) {
            return Err(Error::at(
                self.declaration.node.file,
                0,
                "helper query requires its actual selected derive role",
            ));
        }
        let data = self.data.as_ref().ok_or_else(|| {
            Error::at(
                self.declaration.node.file,
                0,
                "derive helper declaration data is missing",
            )
        })?;
        data.derive_helpers(expected)
    }
    pub(in crate::resolve) fn allowance_at(
        &self,
        index: usize,
        attribute: Attribute,
    ) -> Result<Option<Allowance>, Error> {
        let error = |message| Error::at(self.declaration.node.file, attribute.span.start, message);
        let Some(data) = &self.data else {
            return Ok(None);
        };
        self.package
            .macro_context()
            .map_err(error)?
            .ok_or_else(|| error("missing macro header provider"))?
            .contracts()
            .charge(
                1 + data.witnesses.len().checked_ilog2().unwrap_or(0) as usize,
                0,
            )
            .map_err(error)?;
        let Ok(found) = data
            .witnesses
            .binary_search_by_key(&index, |witness| witness.index)
        else {
            return Ok(None);
        };
        let witness = &data.witnesses[found];
        if witness.attribute != attribute {
            return Err(error("macro allowance attribute identity changed"));
        }
        Ok(witness.allowance.then_some(Allowance {
            declaration: self.declaration.node,
            attribute,
        }))
    }
}

pub(super) fn retain(data: HeaderData, work: &mut State) -> Result<Entry, &'static str> {
    work.charge(1, 4 * size_of::<HeaderData>())?;
    work.payload(data.payload())?;
    Ok(Entry::Owned(Rc::new(data)))
}
