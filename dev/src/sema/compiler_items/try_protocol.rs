//! Real tagged declarations supply protocol identities, never synthetic bodies.
mod declared_calls;
mod flow_variants;
use crate::{
    resolve::{
        bindings::Declaration, expansion::Report, model::DefId, package::Error, unit::NodeKey,
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
    site: NodeKey,
}
impl Query<'_> {
    fn error(&self, message: &'static str) -> Error {
        Declarations::error(self.report, self.site, message)
    }
    fn claimed(&self, role: LangItem) -> Result<Declaration, Error> {
        header_attributes::charge_source_contract_query(self.facts, 1)?;
        let catalog = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error("source Try requires its actual provider catalog"))?;
        catalog
            .require_provider_report(self.report)
            .map_err(|e| self.error(e))?;
        let declaration = catalog.language(role).ok_or_else(|| {
            self.error("source Try requires its actual tagged protocol declarations")
        })?;
        if self
            .report
            .resolution
            .source_bindings
            .as_deref()
            .and_then(|reserved| reserved.language(role))
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
        .and_then(|catalog| catalog.language(LangItem::TryTrait))
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
    query.from_output(&from_output, output, self_type)?;
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
