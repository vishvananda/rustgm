//! Immutable imported originals share one owner; no row is a Checked or Permit.
mod decoded_bodies;
mod interface_goals;
mod invocations;
mod operations;
mod rebuilt_inputs;
mod rebuilt_operations;
use crate::{
    body::{regions::Contract, templates::Template},
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
    support::{crates, hash::HashMap},
};
use std::rc::Rc;
pub(crate) struct Original {
    pub template: Template,
    pub contract: Contract,
}
pub(crate) struct ImportedOriginals {
    rows: Vec<Original>,
    index: HashMap<DefId, usize>,
    steps: usize,
    payload: usize,
}
impl ImportedOriginals {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Original> {
        self.rows.iter()
    }
    pub(crate) fn get(&self, definition: DefId) -> Option<&Original> {
        self.index
            .get(&definition)
            .and_then(|&index| self.rows.get(index))
    }
    pub(crate) fn at(&self, index: usize) -> Option<&Original> {
        self.rows.get(index)
    }
    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }
    pub(crate) fn steps(&self) -> usize {
        self.steps
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(crate::support::retained::vector(&self.rows))
            .saturating_add(crate::support::retained::map(&self.index))
            .saturating_add(self.payload)
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > crates::MAX_COMPONENTS {
            return Err("imported original construction work budget exceeded");
        }
        Ok(())
    }
    fn retain(&mut self, row: Original) -> Result<(), &'static str> {
        let crate::body::ir::Owner::Function { definition, .. } = row.template.body.owner else {
            return Err("imported original lost its function owner");
        };
        if self.index.insert(definition, self.rows.len()).is_some() {
            return Err("duplicate imported executable original");
        }
        self.payload = self
            .payload
            .saturating_add(row.template.body.allocation_bytes())
            .saturating_add(row.template.destruction.allocation_bytes())
            .saturating_add(interface_goals::allocation(&row.contract));
        self.rows.push(row);
        Ok(())
    }
    fn prepare(
        &mut self,
        rows: &crates::generics::Rows,
        available: usize,
    ) -> Result<(), &'static str> {
        self.charge(rows.functions.len().saturating_add(rows.originals.len()))?;
        let mut growth = rows
            .functions
            .len()
            .saturating_mul(4 * (size_of::<Original>() + size_of::<(DefId, usize)>()));
        for body in &rows.originals {
            let ports = body
                .locals
                .len()
                .saturating_add(body.values.len())
                .saturating_add(1);
            let obligations = ports
                .saturating_add(body.values.len())
                .saturating_add(body.blocks.len());
            let header = &rows.functions[body.body.0 as usize];
            let formals = (header.parameters.end - header.parameters.start) as usize;
            let clauses = (header.clauses.end - header.clauses.start) as usize;
            if let Some(calls) = &rows.calls {
                for value in &body.values {
                    self.charge(1)?;
                    let crates::generics::ValueKind::Call {
                        callee, arguments, ..
                    } = value.kind
                    else {
                        continue;
                    };
                    let crates::generics::Type::FunctionItem(app) =
                        rows.types[body.values[callee as usize].ty.0 as usize]
                    else {
                        return Err("call allocation lost its exact application");
                    };
                    let app = &calls.applications[app.0 as usize];
                    let target = &rows.functions[app.target.0 as usize];
                    let formals = (app.arguments.end - app.arguments.start) as usize;
                    let bounds = (target.clauses.end - target.clauses.start) as usize;
                    let inputs = (arguments.end - arguments.start) as usize;
                    growth = growth
                        .saturating_add(
                            4 * (formals + inputs + 1)
                                .saturating_mul(size_of::<crate::body::regions::Obligation>()),
                        )
                        .saturating_add(
                            4 * (formals + bounds).saturating_mul(size_of::<(
                                crate::body::ir::ProofSite,
                                crate::types::parameters::predicates::Predicate,
                            )>()),
                        );
                }
            }
            growth = growth
                .saturating_add(4 * operations::allocation(body))
                .saturating_add(
                    4 * obligations.saturating_mul(size_of::<crate::body::regions::Obligation>()),
                )
                .saturating_add(
                    4 * ports.saturating_mul(
                        size_of::<(
                            crate::body::ir::ProofSite,
                            crate::types::parameters::predicates::Predicate,
                        )>() + size_of::<bool>(),
                    ),
                )
                .saturating_add(
                    4 * formals.saturating_mul(
                        size_of::<crate::types::parameters::ParameterId>()
                            + size_of::<crate::body::regions::Obligation>()
                            + size_of::<(
                                crate::body::ir::ProofSite,
                                crate::types::parameters::predicates::Predicate,
                            )>(),
                    ),
                )
                .saturating_add(
                    4 * clauses
                        .saturating_mul(size_of::<crate::types::parameters::clauses::Clause>()),
                )
                .saturating_add(
                    size_of::<crate::body::regions::BuiltinRequirements>() + 2 * size_of::<usize>(),
                );
        }
        if self.retained_bytes().saturating_add(growth) > crates::MAX_BYTES.min(available) {
            return Err("imported original construction storage budget exceeded");
        }
        Ok(())
    }
}
/// Exact source-only input correspondence. The caller owns failed-query accounting.
pub(crate) fn validate_source_parts(
    facts: &Declarations,
    body: &crate::body::ir::Body,
    contract: &Contract,
    steps: &mut usize,
) -> Result<bool, &'static str> {
    let crate::body::ir::Owner::Function { definition, .. } = body.owner else {
        return Err("source original requires its actual imported function owner");
    };
    if !crate::sema::crates::is_source_original(facts, definition, steps)? {
        return Ok(false);
    }
    let view = crate::sema::crates::source_original(facts, definition, steps)?
        .ok_or("source original cannot execute an absent required method")?;
    rebuilt_operations::validate(&view, body, steps)?;
    rebuilt_inputs::validate(facts, &view, body, contract, steps)?;
    Ok(true)
}

pub(crate) fn build(
    report: &Report,
    facts: &mut Declarations,
    scratch: &mut crate::body::promote::Scratch,
) -> Result<Option<Rc<ImportedOriginals>>, Error> {
    let Some(imported) = report.resolution.crates.as_deref() else {
        return Ok(None);
    };
    if !imported
        .dependencies()
        .artifacts()
        .iter()
        .any(|a| a.generic_payload().is_some())
    {
        return Ok(None);
    }
    let catalog = imported
        .origin_catalog()
        .map_err(|e| report.body_optional_error(None, e))?
        .ok_or_else(|| {
            report.body_optional_error(None, "imported originals have no current catalog")
        })?;
    let (header_work, header_bound, _, _) = facts
        .crates
        .as_deref()
        .and_then(crate::sema::crates::Linkage::original_work)
        .ok_or_else(|| {
            report.body_optional_error(None, "imported originals lack completed header accounting")
        })?;
    if header_work > crates::MAX_COMPONENTS || header_bound > crates::MAX_BYTES {
        return Err(
            report.body_optional_error(None, "imported header accounting exceeds its budget")
        );
    }
    let mut originals = ImportedOriginals {
        rows: Vec::new(),
        index: HashMap::default(),
        steps: 0,
        payload: 0,
    };
    let result = (|| {
        let available = (64_usize * 1024 * 1024).saturating_sub(facts.constant_work.template_bytes);
        for artifact in imported.dependencies().artifacts() {
            let Some(payload) = artifact.generic_payload() else {
                continue;
            };
            let context = Import {
                report,
                imported,
                catalog: &catalog,
                owner: artifact.identity(),
                payload,
            };
            if let crates::generics::PayloadView::Source(source) = payload.view() {
                decoded_bodies::append(
                    &context,
                    facts,
                    scratch,
                    &mut originals,
                    decoded_bodies::Allowance {
                        header_work,
                        header_bytes: header_bound,
                        templates: available,
                    },
                    source.descriptor().rows(),
                )?;
                continue;
            }
            let rows = payload
                .legacy_rows()
                .map_err(|e| report.body_optional_error(None, e))?;
            originals
                .prepare(rows, available)
                .map_err(|e| report.body_optional_error(None, e))?;
            for (function, raw) in rows.functions.iter().zip(&rows.originals) {
                let row = row(&context, facts, function, raw, scratch, &mut originals)?;
                let source = row.template.body.source;
                originals
                    .retain(row)
                    .map_err(|e| report.body_error(source, e))?;
            }
        }
        Ok::<_, Error>(())
    })();
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(originals.steps());
    result?;
    Ok(Some(Rc::new(originals)))
}
use std::sync::Arc;

struct Import<'a> {
    report: &'a Report,
    imported: &'a crate::resolve::crates::Imported,
    catalog: &'a crate::resolve::body_origin::Catalog,
    owner: &'a crates::CrateIdentity,
    payload: &'a crates::generics::Payload,
}
fn row(
    context: &Import<'_>,
    facts: &mut Declarations,
    function: &crates::generics::Function,
    original: &crates::generics::Original,
    scratch: &mut crate::body::promote::Scratch,
    originals: &mut ImportedOriginals,
) -> Result<Original, Error> {
    let report = context.report;
    let definition = context
        .imported
        .generic_definition(context.owner, function.declaration)
        .map_err(|e| report.body_optional_error(None, e))?;
    let header = facts
        .function_header(definition)
        .map_err(|e| report.body_optional_error(None, e))?
        .ok_or_else(|| {
            report.body_optional_error(None, "missing imported original declaration header")
        })?;
    let environment = header
        .environment
        .cloned()
        .ok_or_else(|| report.body_optional_error(None, "missing imported original environment"))?;
    let source = header
        .body
        .ok_or_else(|| report.body_optional_error(None, "imported original body is absent"))?;
    let signature = header.signature;
    let types = facts
        .crates
        .as_ref()
        .ok_or_else(|| report.body_error(source, "missing imported original linkage"))?
        .original_types(context.owner)
        .map_err(|e| report.body_error(source, e))?;
    let operations = operations::Context {
        imported: context.imported,
        catalog: context.catalog,
        owner: context.owner,
        types,
        linkage: facts
            .crates
            .as_ref()
            .ok_or_else(|| report.body_error(source, "missing imported linkage"))?,
    };
    let mut body = operations::build(
        &operations,
        original,
        definition,
        signature,
        function.constant,
        originals,
    )
    .map_err(|e| report.body_error(source, e))?;
    let input = interface_goals::build(facts, context.payload, function, &body, originals)
        .map_err(|e| report.body_error(source, e))?;
    // Scoped destruction is computed from the exact declaration assumptions.
    let needed = crate::types::destruction::NeedsDrop::new(
        facts.destructors.implementations().iter().map(|i| i.item),
    );
    let destruction = crate::body::drops::requirements(
        facts,
        &body,
        &mut needed.scoped(Arc::clone(&environment)),
    )
    .map_err(|e| report.body_error(source, e))?;
    crate::body::promote::instance_reusing(report, facts, &mut body, &destruction, scratch)?;
    crate::body::constants::eligibility_charged(report, facts, &body)?;
    let template = Template {
        body,
        destruction,
        environment,
    };
    Ok(Original {
        template,
        contract: input,
    })
}
