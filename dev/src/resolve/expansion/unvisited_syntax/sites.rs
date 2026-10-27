//! One sparse-site walk separates source authentication from warning retention.
use super::{Error, Failure, FeatureName, FileId, Kind, Package, failure};
use crate::{
    resolve::unit::{NodeKey, Unit},
    syntax::{
        gate_sites::{Declaration, Declarations, Expression},
        token_tree::Span,
    },
};
#[derive(Clone, Copy)]
pub(super) enum Site<'a> {
    Item(&'a Declaration),
    Expression(u32, &'a Expression),
}
impl Site<'_> {
    pub(super) fn span(self) -> Span {
        match self {
            Self::Item(site) => site.span,
            Self::Expression(_, site) => site.keyword,
        }
    }
    pub(super) fn node(self) -> crate::syntax::ast::NodeId {
        match self {
            Self::Item(site) => site.node,
            Self::Expression(_, site) => site.node,
        }
    }
    pub(super) fn context(self) -> Option<(std::num::NonZeroU32, std::num::NonZeroU32)> {
        match self {
            Self::Item(site) => site.context_key().map(|context| (context, context)),
            Self::Expression(_, site) => site.context_key(),
        }
    }
}
pub(super) fn prepare(
    package: &Package,
    file: FileId,
    unit: &Unit,
    entries: &Declarations,
    site: Site<'_>,
    allows: &mut impl FnMut(FileId, Site<'_>, &FeatureName<'_>) -> Result<bool, Error>,
) -> Result<Option<&'static str>, Failure> {
    let error = |error| failure(entries, error);
    match site {
        Site::Item(site) => {
            if matches!(site.kind, Kind::ForeignType | Kind::FinalFunction) {
                return Ok(None);
            }
            crate::resolve::default_items::checked(
                package,
                NodeKey {
                    file,
                    node: site.node,
                },
                site,
            )
            .map_err(error)?;
        }
        Site::Expression(ordinal, expression) => {
            crate::resolve::gate_provenance::checked_expression(
                package,
                file,
                ordinal,
                *expression,
            )
            .map_err(error)?;
            // Unlike try blocks, const closures retain a hard parser-collected
            // gate after cfg removal. Their validator owns that error.
            if expression
                .kind(&unit.configured.ast)
                .map_err(|found| error(Error::at(file, found.offset, found.message)))?
                != crate::syntax::gate_sites::ExpressionKind::TryBlock
            {
                return Ok(None);
            }
        }
    }
    if unit.configured.active[site.node().get() as usize] {
        return Ok(None);
    }
    let (name, enabled, message) = match site {
        Site::Expression(..) => (
            "try_blocks",
            package.requested_try_blocks(),
            "`try` blocks are unstable",
        ),
        Site::Item(declaration) if declaration.kind == Kind::TraitAlias => (
            "trait_alias",
            package.requested_alias_feature(false),
            super::ALIAS_MESSAGE,
        ),
        Site::Item(declaration) => (
            "specialization",
            package.requested_defaultness(declaration.kind == Kind::Function),
            super::MESSAGE,
        ),
    };
    if enabled.map_err(|message| error(Error::at(file, site.span().start, message)))? {
        return Ok(None);
    }
    let feature = FeatureName::new(name).map_err(|message| error(Error::unlocated(message)))?;
    if allows(file, site, &feature).map_err(error)? {
        return Ok(None);
    }
    if matches!(site,Site::Item(declaration) if declaration.kind==Kind::Function) {
        let minimal = FeatureName::new("min_specialization")
            .map_err(|message| error(Error::unlocated(message)))?;
        if allows(file, site, &minimal).map_err(error)? {
            return Ok(None);
        }
    }
    Ok(Some(message))
}
