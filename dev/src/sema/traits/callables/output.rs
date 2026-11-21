//! A callable output equation can expose existing holes while its trait proof stays pending.
use super::{Constructor, Context, Declarations, TermId, infer, open_signature};
use crate::types::{infer::probe::Probe, lifetime_proof::Obligations, substitute::Bound};

/// The callable premise of a selected implementation (`premise`) may expose a
/// closure's output before its projected input is normalized; a projection a
/// body writes waits for that input as before.
pub(in crate::sema) fn symbolic(
    facts: &mut Declarations,
    source: &mut Context,
    root: TermId,
    first: u32,
    premise: bool,
) -> Result<Option<(TermId, Obligations)>, String> {
    let (mut probe, roots) = Probe::new(&facts.types, source, &[root]).map_err(error)?;
    let result = expose(facts, source, &mut probe, roots[0], (first, premise));
    source.account_work(probe.context().steps).map_err(error)?;
    result
}

fn expose(
    facts: &mut Declarations,
    source: &mut Context,
    probe: &mut Probe,
    root: TermId,
    (first, premise): (u32, bool),
) -> Result<Option<(TermId, Obligations)>, String> {
    let mut opening = Bound::open_regions(first)?;
    let root = probe
        .context()
        .map_regions(&mut facts.types, root, &mut opening)
        .map_err(error)?;
    let application = probe
        .context()
        .projection_application(&facts.types, root)
        .map_err(error)?;
    let mut proof = Obligations::new(first);
    proof.regions.end = opening
        .next_region()
        .ok_or("missing callable output region interval")?;
    proof.equalities.extend(
        opening
            .opened_regions()
            .map(|(original, free)| (free, original)),
    );
    proof
        .equalities
        .sort_unstable_by_key(|&(free, original)| (free.0, original.0));
    let opened = open_signature(
        facts,
        probe.context(),
        &application.predicate,
        proof.regions.end,
    )
    .and_then(|opened| {
        match opened.equate_inputs(&mut facts.types, probe.context()) {
            Ok(()) => Ok(opened.signature),
            // A declared generic function's result can determine its own item
            // arguments before projecting its input types. Closure signatures
            // can still be inferred from their input types and defining body.
            Err(infer::Error::Unresolved | infer::Error::Normalization)
                if can_expose_function_output(
                    &mut facts.types,
                    probe.context(),
                    application.predicate.subject(),
                    &opened.signature,
                )? =>
            {
                Ok(opened.signature)
            }
            // A closure signature without its own lifetime binder has one
            // output for every input it accepts. An input written as a
            // projection that is not normalized yet, `I::Item` in
            // `F: FnMut(I::Item) -> Option<B>`, leaves that output exact; the
            // callable premise itself retains the input equation.
            Err(infer::Error::Normalization)
                if premise
                    && opened.signature.regions.is_empty()
                    && matches!(
                        probe
                            .context()
                            .head(&facts.types, application.predicate.subject())?,
                        Some(Constructor::Closure(..))
                    ) =>
            {
                Ok(opened.signature)
            }
            Err(error) => Err(error),
        }
    });
    let output = match opened {
        Ok(matched) => {
            proof.regions.end = matched.regions.end;
            matched.output
        }
        // A declared opaque callable has no signature. Its unique published
        // output equation is forward normalization over settled inputs only.
        Err(infer::Error::Mismatch) => {
            let Some((output, regions)) = crate::sema::traits::results::infer_output(
                facts,
                probe.context(),
                &application.predicate,
                application.item,
                proof.regions.end,
            )?
            else {
                return Ok(None);
            };
            proof.regions.end = regions.end;
            output
        }
        Err(infer::Error::Unresolved | infer::Error::Occurs | infer::Error::Normalization) => {
            return Ok(None);
        }
        Err(e) => return Err(error(e)),
    };
    if !probe.preserves_inputs(source).map_err(error)? {
        return Ok(None);
    }
    proof
        .equalities
        .append(&mut probe.context().region_equalities);
    let mut restoration = proof.restoration()?;
    let output = probe
        .context()
        .map_regions(&mut facts.types, output, &mut restoration)
        .map_err(error)?;
    Ok(probe
        .export(&facts.types, source, output)
        .map_err(error)?
        .map(|output| (output, proof)))
}

/// An unresolved function item can expose a flexible output independent of its
/// own late-bound lifetimes. A method such as `for<'r> fn(&'r S) -> &'r T`
/// gets its output lifetime from the actual input.
fn can_expose_function_output(
    types: &mut crate::types::Store,
    context: &mut Context,
    subject: TermId,
    signature: &super::Matched,
) -> Result<bool, infer::Error> {
    if !matches!(
        context.head(types, subject)?,
        Some(Constructor::FunctionItemArgs(..))
    ) || context.inputs_fixed(types, signature.output)?
    {
        return Ok(false);
    }
    if signature.regions.is_empty() {
        return Ok(true);
    }
    let mut observed = Bound::observe_regions();
    context.map_regions(types, signature.output, &mut observed)?;
    context.account_work(observed.steps)?;
    Ok(!observed
        .observed_regions()
        .any(|region| signature.regions.contains(&region.0)))
}

fn error(error: infer::Error) -> String {
    error.to_string()
}
