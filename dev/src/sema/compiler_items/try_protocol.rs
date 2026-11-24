//! Real tagged declarations supply protocol identities, never synthetic bodies.
mod declared_calls;
mod flow_variants;
use super::{Catalog, Role};
use crate::{
    resolve::{
        body_origin::DeclarationOrigin, expansion::Report, model::DefId, package::Error,
        unit::NodeKey,
    },
    sema::{header_attributes, members::VariantId, model::Declarations},
    support::bindings::LangItem,
    types::{ItemId, TypeId},
};

/// Private construction requires current source headers and actual enum fields.
pub(in crate::sema) struct Protocol {
    pub from_output: DefId,
    pub branch: DefId,
    pub from_residual: DefId,
    pub into_try_type: DefId,
    pub control_flow: ItemId,
    pub continue_: VariantId,
    pub break_: VariantId,
}
struct Query<'a> {
    report: &'a Report,
    facts: &'a Declarations,
    site: Option<NodeKey>,
}
impl Query<'_> {
    fn error(&self, message: &'static str) -> Error {
        self.site.map_or_else(
            || Error::unlocated(message),
            |site| Declarations::error(self.report, site, message),
        )
    }
    fn claimed(&self, role: LangItem) -> Result<Role, Error> {
        let catalog = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error("source Try requires its actual provider catalog"))?;
        header_attributes::charge_source_contract_query(self.facts, 1)?;
        catalog
            .require_provider_report(self.report)
            .map_err(|e| self.error(e))?;
        let declaration = catalog
            .role(role)
            .map_err(|error| self.error(error))?
            .ok_or_else(|| {
                self.error("source Try requires its actual tagged protocol declarations")
            })?;
        if matches!(declaration.origin, DeclarationOrigin::Source(_))
            && self
                .report
                .resolution
                .source_bindings
                .as_deref()
                .and_then(|reserved| reserved.language(role))
                .map(Role::from)
                != Some(declaration)
        {
            return Err(self.error("source Try claim changed its actual reserved declaration"));
        }
        Ok(declaration)
    }
    fn current_header(&self, definition: DefId, origin: NodeKey) -> Result<(), Error> {
        let image = header_attributes::completed_source_image(self.facts, self.report, definition)?;
        if !image.current(self.report.package.unit(origin.file), origin.node) {
            return Err(self.error("source Try declaration changed its source incarnation"));
        }
        Ok(())
    }
}

pub(in crate::sema) fn source(
    report: &Report,
    facts: &Declarations,
    site: NodeKey,
) -> Result<Protocol, Error> {
    checked(report, facts, Some(site))
}

fn checked(
    report: &Report,
    facts: &Declarations,
    site: Option<NodeKey>,
) -> Result<Protocol, Error> {
    let query = Query {
        report,
        facts,
        site,
    };
    let from_output = query.method(LangItem::TryFromOutput)?;
    let branch = query.method(LangItem::TryBranch)?;
    let trait_ = from_output
        .declaration
        .parent
        .ok_or_else(|| query.error("source Try from_output has no actual trait parent"))?;
    if branch.declaration.parent != Some(trait_) {
        return Err(query.error("source Try methods have different actual trait parents"));
    }
    if let Some(tagged) = facts
        .source_bindings
        .as_deref()
        .map(|catalog| catalog.role(LangItem::TryTrait))
        .transpose()
        .map_err(|error| query.error(error))?
        .flatten()
        && tagged.definition != trait_
    {
        return Err(query.error("source Try tag disagrees with its actual method parent"));
    }
    let (output, residual, self_type) =
        query.trait_members(trait_, from_output.declaration, branch.declaration)?;
    if branch.input != self_type {
        return Err(query.error("source Try branch requires its actual by-value Self input"));
    }
    let variants = query.variants(branch.result, residual, output, self_type)?;
    query.output_signature(&from_output, output, self_type)?;
    let from_residual = query.method(LangItem::TryFromResidual)?;
    query.residual_method(&from_residual)?;
    let into_try_type = query.method(LangItem::IntoTryType)?;
    query.residual_conversion(&into_try_type, from_residual.declaration)?;
    Ok(Protocol {
        from_output: from_output.declaration.definition,
        branch: branch.declaration.definition,
        from_residual: from_residual.declaration.definition,
        into_try_type: into_try_type.declaration.definition,
        control_flow: variants.0,
        continue_: variants.1,
        break_: variants.2,
    })
}

/// Source headers and imported headers use the same typed protocol check.
pub(super) fn completed(report: &Report, facts: &Declarations, catalog: &Catalog) -> bool {
    catalog
        .language(LangItem::TryTrait)
        .is_some_and(|trait_| source(report, facts, trait_.origin).is_ok())
}

pub(super) fn imported(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
) -> Result<bool, &'static str> {
    let mut count = 0;
    for role in LangItem::TRY_PROTOCOL {
        *steps = steps.saturating_add(1);
        count += usize::from(catalog.role(role)?.is_some());
    }
    if count == 0 {
        return Ok(false);
    }
    if count != LangItem::TRY_PROTOCOL.len() {
        return Err("imported Try requires its complete actual protocol roles");
    }
    *steps = steps.saturating_add(48);
    checked(report, facts, None)
        .map_err(|_| "imported Try requires its exact checked callable and variant declarations")?;
    Ok(true)
}
